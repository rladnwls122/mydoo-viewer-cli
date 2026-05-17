use std::io::Read;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub fn extract_text(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut section_names: Vec<String> = archive
        .file_names()
        .filter(|n| n.starts_with("Contents/section") && n.ends_with(".xml"))
        .map(|s| s.to_string())
        .collect();
    section_names.sort();
    if section_names.is_empty() {
        return Err("hwpx 안에 Contents/section*.xml이 없음".into());
    }

    let mut out = String::new();
    for name in section_names {
        let mut entry = archive.by_name(&name).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        let text = parse_section(&buf)?;
        out.push_str(&text);
        out.push('\n');
    }
    Ok(out)
}

fn parse_section(xml: &[u8]) -> Result<String, String> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut out = String::new();
    let mut buf = Vec::new();
    let mut in_t = false;
    let mut depth_t = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(format!("XML parse error: {}", e)),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                if local == b"t" {
                    in_t = true;
                    depth_t += 1;
                } else if local == b"linesegarray" {
                    // skip
                } else if local == b"p" {
                    out.push('\n');
                }
            }
            Ok(Event::End(e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                if local == b"t" {
                    if depth_t > 0 {
                        depth_t -= 1;
                    }
                    if depth_t == 0 {
                        in_t = false;
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if in_t {
                    let s = t.unescape().map_err(|e| e.to_string())?;
                    out.push_str(&s);
                }
            }
            Ok(Event::Empty(_)) => {}
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

fn local_name(qname: &[u8]) -> &[u8] {
    if let Some(pos) = qname.iter().position(|&b| b == b':') {
        &qname[pos + 1..]
    } else {
        qname
    }
}
