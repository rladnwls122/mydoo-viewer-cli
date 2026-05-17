use std::io::Read;
use std::path::Path;

use flate2::read::DeflateDecoder;

pub fn extract_text(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut comp = cfb::CompoundFile::open(file).map_err(|e| e.to_string())?;

    let compressed = is_compressed(&mut comp)?;

    // 모든 스트림 path 수집 — 폴백 + 진단용
    let all_paths: Vec<String> = comp
        .walk()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    // 1) 본문 Section 스트림 — 케이스/슬래시/위치 변형 다 잡기
    let mut section_paths: Vec<String> = all_paths
        .iter()
        .filter(|p| {
            let lower = p.to_lowercase();
            (lower.contains("bodytext") || lower.contains("viewtext"))
                && lower.contains("section")
        })
        .cloned()
        .collect();
    section_paths.sort();

    if !section_paths.is_empty() {
        let mut out = String::new();
        for path in section_paths {
            let mut stream = comp.open_stream(&path).map_err(|e| e.to_string())?;
            let mut raw = Vec::new();
            stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
            let bytes = if compressed { inflate(&raw)? } else { raw };
            let text = parse_section_records(&bytes);
            out.push_str(&text);
            out.push('\n');
        }
        return Ok(out);
    }

    // 2) PrvText 폴백 — 거의 모든 hwp에 있는 미리보기 텍스트 (UTF-16LE 평문)
    if let Some(prv_path) = all_paths
        .iter()
        .find(|p| p.to_lowercase().ends_with("prvtext"))
    {
        let mut stream = comp.open_stream(prv_path).map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        let text = decode_utf16_le_local(&raw);
        return Ok(format!(
            "{}\n\n— (본문 Section 스트림을 찾지 못해 미리보기 텍스트만 추출했습니다. 전체 본문은 한컴오피스로 열어 주세요.)",
            text
        ));
    }

    // 3) 진단 — 실제 스트림 목록을 같이 노출
    Err(format!(
        "hwp 안에서 본문(BodyText/Section, ViewText/Section)도 미리보기(PrvText)도 찾지 못했습니다.\n\n스트림 목록:\n  {}",
        all_paths.join("\n  ")
    ))
}

fn decode_utf16_le_local(bytes: &[u8]) -> String {
    let mut units = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i + 1 < bytes.len() {
        let u = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        if u == 0 {
            break;
        }
        units.push(u);
        i += 2;
    }
    String::from_utf16_lossy(&units)
}

fn is_compressed(comp: &mut cfb::CompoundFile<std::fs::File>) -> Result<bool, String> {
    let mut stream = comp
        .open_stream("/FileHeader")
        .map_err(|e| format!("FileHeader 누락: {}", e))?;
    let mut header = [0u8; 256];
    let n = stream.read(&mut header).map_err(|e| e.to_string())?;
    if n < 36 {
        return Err("FileHeader 너무 짧음".into());
    }
    // Property bitfield는 36바이트 오프셋부터 4바이트(LE). bit0이 compressed 여부.
    let prop = u32::from_le_bytes([header[36], header[37], header[38], header[39]]);
    Ok(prop & 0x1 != 0)
}

fn inflate(bytes: &[u8]) -> Result<Vec<u8>, String> {
    // HWP 5.0 본문은 보통 raw deflate. 일부 변형은 zlib wrapper(0x78 ...) 사용.
    // 둘 다 시도하고, 도중에 깨져도 그때까지 풀린 만큼은 반환한다.
    let is_zlib = bytes.len() >= 2 && bytes[0] == 0x78;

    fn drain<R: std::io::Read>(reader: &mut R) -> (Vec<u8>, Option<String>) {
        let mut out = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => return (out, None),
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => return (out, Some(e.to_string())),
            }
        }
    }
    let try_decode = |use_zlib: bool| -> (Vec<u8>, Option<String>) {
        if use_zlib {
            drain(&mut flate2::read::ZlibDecoder::new(bytes))
        } else {
            drain(&mut DeflateDecoder::new(bytes))
        }
    };

    // 1) 헤더로 추정한 방식 먼저 시도
    let (mut out, err1) = try_decode(is_zlib);
    if err1.is_none() {
        return Ok(out);
    }
    // 2) 반대 방식으로 재시도
    let (out2, err2) = try_decode(!is_zlib);
    if err2.is_none() {
        return Ok(out2);
    }
    // 둘 다 도중에 깨진 경우 — 더 많이 풀린 쪽을 부분 결과로 반환
    if out2.len() > out.len() {
        out = out2;
    }
    if out.is_empty() {
        return Err(format!(
            "hwp 압축 해제 실패: {}",
            err1.unwrap_or_else(|| err2.unwrap_or_default())
        ));
    }
    // 부분 추출이라도 반환
    Ok(out)
}

// HWP 5.0 본문 레코드 — 4바이트 헤더(tag_id 10bit, level 10bit, size 12bit).
// PARA_TEXT(67)의 데이터에서 UTF-16LE 문자만 추려 텍스트 추출.
// (제어문자 1~31 중 일부는 inline-object placeholder — 단순히 스킵)
fn parse_section_records(bytes: &[u8]) -> String {
    const PARA_TEXT: u32 = 67;
    let mut out = String::new();
    let mut i = 0usize;
    while i + 4 <= bytes.len() {
        let h = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        let tag_id = h & 0x3FF;
        let _level = (h >> 10) & 0x3FF;
        let mut size = ((h >> 20) & 0xFFF) as usize;
        i += 4;
        if size == 0xFFF {
            if i + 4 > bytes.len() {
                break;
            }
            size = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
            i += 4;
        }
        if i + size > bytes.len() {
            break;
        }
        if tag_id == PARA_TEXT {
            let data = &bytes[i..i + size];
            extract_para_text(data, &mut out);
            out.push('\n');
        }
        i += size;
    }
    out
}

fn extract_para_text(data: &[u8], out: &mut String) {
    let mut i = 0usize;
    while i + 2 <= data.len() {
        let ch = u16::from_le_bytes([data[i], data[i + 1]]);
        i += 2;
        match ch {
            // 인라인 컨트롤 (1,2,3,4,5,6,7,8,11,12,14,15,16,17,18,19,21,22,23,24,28,29,30,31): 14바이트 추가 스킵
            1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 11 | 12 | 14 | 15 | 16 | 17 | 18 | 19 | 21 | 22 | 23
            | 24 | 28 | 29 | 30 | 31 => {
                i += 14;
            }
            9 => out.push('\t'),
            10 | 13 => out.push('\n'),
            _ => {
                if let Some(c) = char::from_u32(ch as u32) {
                    out.push(c);
                }
            }
        }
    }
}
