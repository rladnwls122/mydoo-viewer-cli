# 마이두 뷰어 (mydoo-viewer)

> 마이두 패밀리의 가장 가벼운 문서 뷰어 — `.md` · `.html` · `.txt` · `.docx` · `.hwp` · `.hwpx` 한 창에서.

탐색기에서 더블클릭하면 작은 창 하나가 뜬다. 좌측 사이드바에서 폴더를 열면 그 폴더 안 문서 파일만 트리로 보인다. 본문은 GitHub 스타일로 렌더된다. **AI가 만든 마크다운 문서를 받는 사람에게도 부담 없이 보내고 싶어서** 만들었다 — 받는 사람이 한컴오피스·워드 안 깔려 있어도 이 뷰어 하나면 끝.

## 특징

- **정말 가벼움** — 한 자리수~열 자리수 MB. 한컴오피스 1GB 대비 압도적.
- **만능** — md/html/txt 외에 docx, hwp, hwpx까지 내용 확인 가능.
- **깃 스타일 렌더** — `github-markdown-css` 그대로.
- **사이드바 폴더 트리** — 문서 파일(`.md / .html / .txt / .docx / .hwp / .hwpx`)만 필터링.
- **시스템 다크/라이트 자동** — OS 테마 따라감.
- **윈도우 파일 연결** — 설치 시 위 확장자 더블클릭으로 바로 열림.
- **MIT** — 자유롭게 쓰고 고치고 재배포.

## 지원 포맷

| 확장자 | 렌더 방식 |
|------|--------|
| `.md`, `.markdown` | marked.js + github-markdown-css (GitHub Flavored Markdown) |
| `.html`, `.htm` | DOMPurify 새니타이즈 후 그대로 (스크립트·iframe 차단) |
| `.txt` | 모노스페이스 그대로 |
| `.docx` | mammoth.js로 HTML 변환 |
| `.hwpx` | Rust 직접 파서 — OWPML 텍스트 추출 (v1은 텍스트만, 서식은 v2 예정) |
| `.hwp` | Rust 직접 파서 — HWP 5.0 PARA_TEXT 레코드 텍스트 추출 |

> v1은 "내용 확인용"이 목표라 docx/hwp/hwpx는 텍스트 위주 렌더입니다. 정밀 서식·이미지·표는 v2에서 확장 예정.

## 개발

```bash
# 처음 한 번: Rust 툴체인 + Tauri CLI
winget install Rustlang.Rustup
cargo install tauri-cli --version "^2.0" --locked

# 개발 실행
cd src-tauri && cargo tauri dev

# 릴리즈 빌드 (인스톨러 생성)
cd src-tauri && cargo tauri build
```

빌드 결과: `src-tauri/target/release/`에 `.exe`, `src-tauri/target/release/bundle/nsis/`에 인스톨러.

## 키바인딩

| 키 | 기능 |
|----|------|
| `Ctrl+O` | 파일 열기 |
| `Ctrl+K` | 폴더 열기 |
| `Ctrl+B` | 사이드바 토글 |

## 의존 라이브러리

- **Frontend**: [marked](https://marked.js.org/) · [DOMPurify](https://github.com/cure53/DOMPurify) · [mammoth.js](https://github.com/mwilliamson/mammoth.js) · [github-markdown-css](https://github.com/sindresorhus/github-markdown-css)
- **Backend (Rust)**: [tauri](https://tauri.app/) · [zip](https://crates.io/crates/zip) · [quick-xml](https://crates.io/crates/quick-xml) · [cfb](https://crates.io/crates/cfb) · [encoding_rs](https://crates.io/crates/encoding_rs)

## 라이선스

MIT — [LICENSE](LICENSE)
