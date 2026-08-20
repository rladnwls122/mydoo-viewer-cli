use std::path::{Path, PathBuf};
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
struct Entry {
    name: String,
    path: String,
    is_dir: bool,
}

#[derive(Serialize)]
struct OpenedFile {
    path: String,
    kind: String,
    content: String,
}

fn ext_kind(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "md" | "markdown" => Some("markdown"),
        "html" | "htm" => Some("html"),
        "txt" => Some("text"),
        "docx" => Some("docx"),
        "hwp" => Some("hwp"),
        "hwpx" => Some("hwpx"),
        _ => None,
    }
}

#[tauri::command]
fn list_dir(path: String) -> Result<Vec<Entry>, String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    let mut entries: Vec<Entry> = Vec::new();
    let read = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for item in read.flatten() {
        let p = item.path();
        let name = match item.file_name().to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        if name.starts_with('.') {
            continue;
        }
        let is_dir = p.is_dir();
        if !is_dir {
            // 사이드바에는 마크다운만 노출 — 다른 포맷은 파일 연결/CLI 인자로만 연다.
            if ext_kind(&p) != Some("markdown") {
                continue;
            }
        }
        entries.push(Entry {
            name,
            path: p.to_string_lossy().to_string(),
            is_dir,
        });
    }
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

#[tauri::command]
fn parent_dir(path: String) -> Option<String> {
    PathBuf::from(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn list_drives() -> Vec<Entry> {
    let mut out = Vec::new();
    #[cfg(windows)]
    {
        for letter in b'A'..=b'Z' {
            let p = format!("{}:\\", letter as char);
            if std::path::Path::new(&p).is_dir() {
                out.push(Entry {
                    name: format!("{}:", letter as char),
                    path: p,
                    is_dir: true,
                });
            }
        }
    }
    #[cfg(not(windows))]
    {
        out.push(Entry {
            name: "/".into(),
            path: "/".into(),
            is_dir: true,
        });
    }
    out
}

#[tauri::command]
fn home_dir() -> Option<String> {
    std::env::var("USERPROFILE")
        .ok()
        .or_else(|| std::env::var("HOME").ok())
}

#[tauri::command]
async fn read_file(app: tauri::AppHandle, path: String) -> Result<OpenedFile, String> {
    let p = PathBuf::from(&path);
    let kind = ext_kind(&p).ok_or_else(|| format!("Unsupported file: {}", path))?;
    match kind {
        "markdown" | "html" | "text" => {
            let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
            let content = decode_text(&bytes);
            Ok(OpenedFile {
                path: path.clone(),
                kind: kind.to_string(),
                content,
            })
        }
        "docx" => {
            let bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(OpenedFile {
                path: path.clone(),
                kind: "docx".into(),
                content: b64,
            })
        }
        "hwpx" => {
            let html = call_hwp_sidecar(&app, &path).await?;
            Ok(OpenedFile {
                path: path.clone(),
                kind: kind.to_string(),
                content: html,
            })
        }
        "hwp" => {
            let text = crate::hwp::extract_text(&p).map_err(|e| e.to_string())?;
            Ok(OpenedFile {
                path: path.clone(),
                kind: "hwp".into(),
                content: text,
            })
        }
        _ => Err("Unsupported".into()),
    }
}

async fn call_hwp_sidecar(app: &tauri::AppHandle, file_path: &str) -> Result<String, String> {
    use tauri_plugin_shell::ShellExt;
    let cmd = app
        .shell()
        .sidecar("mydoo-parser")
        .map_err(|e| format!("sidecar 위치 실패: {}", e))?
        .args([file_path]);
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("sidecar 실행 실패: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("parser exit {:?}: {}", output.status.code(), stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("parser JSON 디코드 실패: {}", e))?;
    let html = parsed
        .get("html")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !html.is_empty() {
        return Ok(html.to_string());
    }
    let text = parsed.get("text").and_then(|v| v.as_str()).unwrap_or("");
    Ok(format!("<pre>{}</pre>", text.replace('<', "&lt;").replace('>', "&gt;")))
}

fn decode_text(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return decode_utf16_le(&bytes[2..]);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode_utf16_be(&bytes[2..]);
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // CP949 (한글) 폴백 — euc-kr 호환
            match encoding_rs::EUC_KR.decode(bytes) {
                (cow, _, _) => cow.into_owned(),
            }
        }
    }
}

fn decode_utf16_le(bytes: &[u8]) -> String {
    let mut units = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i + 1 < bytes.len() {
        units.push(u16::from_le_bytes([bytes[i], bytes[i + 1]]));
        i += 2;
    }
    String::from_utf16_lossy(&units)
}

fn decode_utf16_be(bytes: &[u8]) -> String {
    let mut units = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i + 1 < bytes.len() {
        units.push(u16::from_be_bytes([bytes[i], bytes[i + 1]]));
        i += 2;
    }
    String::from_utf16_lossy(&units)
}

#[tauri::command]
async fn open_file_dialog(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .add_filter("문서", &["md", "markdown", "html", "htm", "txt", "docx", "hwp", "hwpx"])
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok().map(|pb| pb.to_string_lossy().to_string()))
}

#[tauri::command]
async fn open_dir_dialog(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok().map(|pb| pb.to_string_lossy().to_string()))
}

#[derive(Serialize, Clone)]
struct InitialPath {
    path: String,
    is_dir: bool,
}

#[tauri::command]
fn initial_file(app: tauri::AppHandle) -> Option<InitialPath> {
    let state = app.state::<InitialArg>();
    state.0.lock().ok().and_then(|g| g.clone())
}

struct InitialArg(std::sync::Mutex<Option<InitialPath>>);

// hwpx는 Node sidecar(mydoo-parser, hwpxjs)가 처리하고,
// hwp(구형)는 Rust 자체 텍스트 추출로 처리한다 (node-hwp가 Bun 런타임 호환 이슈).
mod hwp;

pub fn run() {
    // 'mdv .' 같은 상대 경로는 런처의 cwd 기준으로 절대화한다. canonicalize 가 붙이는
    // Windows \\?\ 접두사는 표시용 경로에서 떼어낸다.
    let initial = std::env::args()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .and_then(|p| {
            let pb = std::fs::canonicalize(&p).ok()?;
            let is_dir = pb.is_dir();
            if !is_dir && ext_kind(&pb).is_none() {
                return None;
            }
            let s = pb.to_string_lossy().to_string();
            let s = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
            Some(InitialPath { path: s, is_dir })
        });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(InitialArg(std::sync::Mutex::new(initial)))
        .invoke_handler(tauri::generate_handler![
            list_dir,
            parent_dir,
            list_drives,
            home_dir,
            read_file,
            open_file_dialog,
            open_dir_dialog,
            initial_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running mydoo-viewer");
}
