use std::io::Read;
use std::path::Path;

use flate2::read::DeflateDecoder;

pub fn extract_text(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut comp = cfb::CompoundFile::open(file).map_err(|e| e.to_string())?;

    let compressed = is_compressed(&mut comp)?;

    let section_paths: Vec<String> = comp
        .walk()
        .filter_map(|entry| {
            let p = entry.path().to_string_lossy().to_string();
            if p.starts_with("/BodyText/Section") || p.starts_with("/ViewText/Section") {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    let mut sorted = section_paths;
    sorted.sort();
    if sorted.is_empty() {
        return Err("hwp 안에 Section 스트림이 없음".into());
    }

    let mut out = String::new();
    for path in sorted {
        let mut stream = comp.open_stream(&path).map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        let bytes = if compressed {
            inflate(&raw)?
        } else {
            raw
        };
        let text = parse_section_records(&bytes);
        out.push_str(&text);
        out.push('\n');
    }
    Ok(out)
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
    let mut decoder = DeflateDecoder::new(bytes);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| format!("hwp 압축 해제 실패: {}", e))?;
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
