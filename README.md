# Mydoo Viewer

> A lightweight universal document viewer for `.md` · `.html` · `.txt` · `.docx` · `.hwp` · `.hwpx`.

---

## Why we built this

Since the early days of personal computing, document formats have been bound to specific operating systems and applications. That fragmentation has persisted to this day.

When AI and developers needed a format that worked freely across every environment, **Markdown** stepped in, and it is now widely adopted.

There was an earlier attempt to settle on a universal document format with **HTML**, but writing HTML by hand was hard for most people, and so individuals rarely used it as a daily document format.

Today, as AI becomes part of everyday life and authors documents on our behalf, **HTML is finally entering the era it was meant for** — with its rich support for layout, visuals, and embedded objects such as images and charts.

An era where presentations are made without PowerPoint.
An era where beautiful documents are written without Word.

**Mydoo Viewer** is a lightweight viewer for this new era.

Mydoo Viewer is released under the **MIT License** — please use it freely. If there is anything you would like to see added, please email **cashmapworkers@gmail.com**. We welcome your suggestions.

---

## 왜 만들었는가

본격적인 컴퓨터 보급 시점에 OS별·프로그램별로 문서 포맷이 만들어졌고, 그 분절된 구조가 오늘날까지 이어져 왔습니다.

AI와 개발자가 어떤 환경에서도 제약 없이 사용할 수 있는 포맷이 필요해지면서 **마크다운(Markdown)** 포맷이 쓰이기 시작했고, 지금도 폭넓게 사용되고 있습니다.

이전 시대에 표준 문서 포맷에 대한 고민이 있었고 **HTML이 표준으로 지정**되었으나, HTML 문서를 직접 작성하기가 어려운 탓에 개인이 일상 문서로 활용하기에는 불편했습니다.

이제 AI가 일상으로 들어오면서 문서 작성을 AI가 대신해 주는 시대가 되었습니다. 그 결과 가시성·가독성이 뛰어나고 이미지·차트 같은 다양한 오브젝트를 풍부하게 표현할 수 있는 **HTML이 비로소 제대로 쓰이는 시대**가 열리고 있습니다.

PowerPoint 없이도 프레젠테이션을 만들 수 있는 시대.
Word 없이도 훌륭한 문서를 만들 수 있는 시대.

**마이두 뷰어(Mydoo Viewer)** 는 그 시대에 어울리는 경량 뷰어입니다.

MIT 라이선스로 공개되어 있으니 마음껏 활용해 주십시오. 추가하고 싶은 기능이나 의견이 있으시면 **cashmapworkers@gmail.com** 으로 메일 부탁드립니다. 의견을 기쁘게 받겠습니다.

---

## Supported formats · 지원 포맷

| Extension | Rendering |
|------|--------|
| `.md`, `.markdown` | marked.js + github-markdown-css (GitHub Flavored Markdown) |
| `.html`, `.htm` | Sanitized via DOMPurify (scripts and iframes blocked) |
| `.txt` | Monospace plain text |
| `.docx` | Converted to HTML via mammoth.js |
| `.hwpx` | Text extraction via Rust ZIP + quick-xml (OWPML) |
| `.hwp` | Text extraction via Rust CFB + HWP 5.0 records |

> v1 focuses on **content viewing**. Rich formatting, images, and tables for `.docx` / `.hwp` / `.hwpx` are scheduled for v2.
> v1은 "내용 확인용"이 목표이며, `.docx` / `.hwp` / `.hwpx` 의 정밀 서식·이미지·표는 v2에서 확장될 예정입니다.

---

## Download · 다운로드

Get the latest installer from the [Releases page](https://github.com/kocoredisk/mydoo-viewer/releases).
최신 인스톨러는 [Releases 페이지](https://github.com/kocoredisk/mydoo-viewer/releases) 에서 받으실 수 있습니다.

---

## Features · 기능

- Sidebar file explorer — only document files (`.md / .html / .txt / .docx / .hwp / .hwpx`) are shown.
- GitHub-style rendering for Markdown and HTML.
- Automatic system dark / light theme, with a manual toggle (persisted via `localStorage`).
- Windows file associations — installed extensions open directly via double-click.
- Keyboard shortcut: `Ctrl + B` to toggle the sidebar.

- 사이드바 파일 탐색기 — 문서 파일(`.md / .html / .txt / .docx / .hwp / .hwpx`)만 노출됩니다.
- 마크다운·HTML을 GitHub 스타일로 렌더링합니다.
- 시스템 다크/라이트 테마 자동 + 수동 토글(상태는 `localStorage`에 저장됩니다).
- 윈도우 파일 연결 — 설치 시 위 확장자들은 더블클릭으로 바로 열립니다.
- 단축키: `Ctrl + B` — 사이드바 토글.

---

## Build from source · 직접 빌드

```bash
# Prerequisite: Rust toolchain + Tauri CLI
winget install Rustlang.Rustup
cargo install tauri-cli --version "^2.0" --locked

# Run in dev mode
cd src-tauri && cargo tauri dev

# Build release (installer + standalone exe)
cd src-tauri && cargo tauri build
```

Artifacts:

- `src-tauri/target/release/mydoo-viewer.exe` — standalone executable
- `src-tauri/target/release/bundle/nsis/Mydoo Viewer_0.1.0_x64-setup.exe` — installer

---

## System requirements · 시스템 요구사항

- Windows 10 / 11 (x64)
- WebView2 Runtime (bundled with Windows 11 by default)

---

## Third-party libraries · 사용 라이브러리

- **Frontend** — [marked](https://marked.js.org/) · [DOMPurify](https://github.com/cure53/DOMPurify) · [mammoth.js](https://github.com/mwilliamson/mammoth.js) · [github-markdown-css](https://github.com/sindresorhus/github-markdown-css)
- **Backend (Rust)** — [tauri](https://tauri.app/) · [zip](https://crates.io/crates/zip) · [quick-xml](https://crates.io/crates/quick-xml) · [cfb](https://crates.io/crates/cfb) · [encoding_rs](https://crates.io/crates/encoding_rs)

---

## License · 라이선스

MIT — see [LICENSE](LICENSE).

## Contact · 연락

Questions, feedback, feature requests: **cashmapworkers@gmail.com**
문의·피드백·기능 요청: **cashmapworkers@gmail.com**
