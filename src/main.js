// Mydoo Viewer — frontend logic
const invoke = window.__TAURI__.core.invoke;
const $ = (sel) => document.querySelector(sel);

const tree = $("#tree");
const viewer = $("#viewer");
const currentFileLabel = $("#current-file");
const sidebarPath = $("#sidebar-path");

let currentDir = null;
let currentFile = null;

marked.setOptions({ gfm: true, breaks: false, headerIds: true, mangle: false });

const FILE_ICON = {
  md: "📝", markdown: "📝",
  html: "🌐", htm: "🌐",
  txt: "📃",
  docx: "📘",
  hwp: "📕", hwpx: "📕",
};
const folderIcon = "📁";
const driveIcon = "💾";

function iconFor(entry) {
  if (entry.is_dir) {
    if (/^[A-Za-z]:\\?$/.test(entry.path)) return driveIcon;
    return folderIcon;
  }
  const ext = entry.name.split(".").pop().toLowerCase();
  return FILE_ICON[ext] || "📄";
}

function renderTree(entries) {
  tree.innerHTML = "";
  for (const e of entries) {
    const li = document.createElement("li");
    li.dataset.path = e.path;
    li.dataset.isDir = e.is_dir;
    const icon = document.createElement("span");
    icon.className = "icon";
    icon.textContent = iconFor(e);
    const label = document.createElement("span");
    label.className = "name";
    label.textContent = e.name;
    li.appendChild(icon);
    li.appendChild(label);
    li.addEventListener("click", () => {
      if (e.is_dir) loadDir(e.path);
      else {
        openFile(e.path);
        markActive(li);
      }
    });
    tree.appendChild(li);
  }
}

async function loadDir(dir) {
  try {
    const entries = await invoke("list_dir", { path: dir });
    currentDir = dir;
    sidebarPath.textContent = dir;
    sidebarPath.title = dir;
    renderTree(entries);
  } catch (err) {
    showError(`폴더를 읽을 수 없음: ${err}`);
  }
}

async function loadDrives() {
  const drives = await invoke("list_drives");
  currentDir = null;
  sidebarPath.textContent = "내 PC";
  sidebarPath.title = "";
  renderTree(drives);
}

function markActive(li) {
  for (const el of tree.querySelectorAll("li.active")) el.classList.remove("active");
  li.classList.add("active");
}

async function openFile(path) {
  try {
    currentFile = path;
    currentFileLabel.textContent = path;
    document.title = `${baseName(path)} — Mydoo Viewer`;
    const file = await invoke("read_file", { path });
    renderFile(file);
  } catch (err) {
    showError(friendlyError(err, path));
  }
}

function friendlyError(err, path) {
  const msg = String(err);
  if (/Corrupted zip|End of data reached/i.test(msg)) {
    return `이 파일은 컨테이너가 손상되어 열 수 없습니다. (원본이 손상되었거나 전송 중 잘려진 가능성)\n파일: ${path}`;
  }
  if (/password|encrypted/i.test(msg)) {
    return `비밀번호로 보호된 문서는 지원되지 않습니다.\n파일: ${path}`;
  }
  if (/Unsupported file/i.test(msg)) {
    return `지원하지 않는 확장자입니다.\n파일: ${path}`;
  }
  return `파일을 열 수 없음: ${msg}`;
}

function renderFile(file) {
  viewer.classList.remove("empty", "plain");
  const { kind, content } = file;

  if (kind === "markdown") {
    const html = marked.parse(content);
    viewer.innerHTML = DOMPurify.sanitize(html, { ADD_ATTR: ["target"] });
    return;
  }
  if (kind === "html") {
    viewer.innerHTML = DOMPurify.sanitize(content, {
      ADD_ATTR: ["target"],
      FORBID_TAGS: ["script", "iframe", "object", "embed", "link"],
      FORBID_ATTR: ["onerror", "onload", "onclick"],
    });
    return;
  }
  if (kind === "text") {
    viewer.classList.add("plain");
    viewer.innerHTML = "";
    const pre = document.createElement("pre");
    pre.textContent = content;
    viewer.appendChild(pre);
    return;
  }
  if (kind === "docx") {
    const bytes = base64ToBytes(content);
    mammoth.convertToHtml({ arrayBuffer: bytes.buffer }).then((result) => {
      viewer.innerHTML = DOMPurify.sanitize(result.value);
    }).catch((e) => showError(`docx 변환 실패: ${e}`));
    return;
  }
  if (kind === "hwpx") {
    // sidecar(hwpxjs)가 만든 HTML을 그대로 렌더 — 표·서식 보존
    viewer.innerHTML = DOMPurify.sanitize(content, {
      ADD_ATTR: ["target"],
      FORBID_TAGS: ["script", "iframe", "object", "embed", "link"],
      FORBID_ATTR: ["onerror", "onload", "onclick"],
    });
    return;
  }
  if (kind === "hwp") {
    // hwp 구형: Rust 텍스트 추출 결과를 모노스페이스로 (서식 v0.3에서 보강)
    viewer.classList.add("plain");
    viewer.innerHTML = "";
    const pre = document.createElement("pre");
    pre.textContent = content;
    viewer.appendChild(pre);
    return;
  }
  showError(`알 수 없는 포맷: ${kind}`);
}

function base64ToBytes(b64) {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes;
}

function baseName(p) {
  return p.replace(/\\/g, "/").split("/").pop();
}

function showError(msg) {
  viewer.classList.remove("empty", "plain");
  viewer.innerHTML = `<div class="error">${escapeHtml(msg)}</div>`;
}

function escapeHtml(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function highlightInTree(path) {
  for (const li of tree.querySelectorAll("li")) {
    if (li.dataset.path === path) {
      markActive(li);
      li.scrollIntoView({ block: "nearest" });
      break;
    }
  }
}

async function openSidebarWithDefault() {
  // 사이드바 열릴 때 기본 진입점 — 홈
  if (!currentDir) {
    const home = await invoke("home_dir");
    if (home) await loadDir(home);
    else await loadDrives();
  }
}

// 헤더 버튼 — 사이드바 토글
$("#btn-toggle-sidebar").addEventListener("click", async () => {
  const closed = document.body.classList.toggle("no-sidebar");
  if (!closed) await openSidebarWithDefault();
});

// 사이드바 내부 버튼
$("#btn-home").addEventListener("click", async () => {
  const home = await invoke("home_dir");
  if (home) loadDir(home);
});

$("#btn-drives").addEventListener("click", () => loadDrives());

$("#btn-up").addEventListener("click", async () => {
  if (!currentDir) return;
  const parent = await invoke("parent_dir", { path: currentDir });
  if (parent && parent !== currentDir) loadDir(parent);
  else loadDrives();
});

// 사이드바 리사이즈 핸들
const MIN_W = 160;
const MAX_W = 600;
const savedW = parseInt(localStorage.getItem("sidebar-w") || "", 10);
if (savedW >= MIN_W && savedW <= MAX_W) {
  document.documentElement.style.setProperty("--sidebar-w", `${savedW}px`);
}
{
  const handle = $("#resize-handle");
  let dragging = false;
  handle.addEventListener("mousedown", (e) => {
    e.preventDefault();
    dragging = true;
    handle.classList.add("dragging");
    document.body.classList.add("resizing");
  });
  document.addEventListener("mousemove", (e) => {
    if (!dragging) return;
    const w = Math.min(MAX_W, Math.max(MIN_W, e.clientX));
    document.documentElement.style.setProperty("--sidebar-w", `${w}px`);
  });
  document.addEventListener("mouseup", () => {
    if (!dragging) return;
    dragging = false;
    handle.classList.remove("dragging");
    document.body.classList.remove("resizing");
    const w = parseInt(
      getComputedStyle(document.documentElement).getPropertyValue("--sidebar-w"),
      10,
    );
    if (w >= MIN_W && w <= MAX_W) localStorage.setItem("sidebar-w", String(w));
  });
}

// 테마 토글
function currentTheme() {
  if (document.body.classList.contains("theme-dark")) return "dark";
  if (document.body.classList.contains("theme-light")) return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}
function applyTheme(theme) {
  document.body.classList.remove("theme-light", "theme-dark");
  if (theme === "light") document.body.classList.add("theme-light");
  else if (theme === "dark") document.body.classList.add("theme-dark");
  $("#btn-theme").textContent = currentTheme() === "dark" ? "☀️" : "🌙";
}
applyTheme(localStorage.getItem("theme"));
$("#btn-theme").addEventListener("click", () => {
  const next = currentTheme() === "dark" ? "light" : "dark";
  localStorage.setItem("theme", next);
  applyTheme(next);
});

// 키바인딩
document.addEventListener("keydown", (e) => {
  const ctrl = e.ctrlKey || e.metaKey;
  if (!ctrl) return;
  if (e.key.toLowerCase() === "b") {
    e.preventDefault();
    $("#btn-toggle-sidebar").click();
  }
});

// 시작: 더블클릭으로 들어온 파일이 있으면 그 부모를 사이드바에 띄우고 파일 연다 (사이드바 자동 펼침)
(async () => {
  const initial = await invoke("initial_file");
  if (initial) {
    document.body.classList.remove("no-sidebar");
    const parent = await invoke("parent_dir", { path: initial });
    if (parent) await loadDir(parent);
    await openFile(initial);
    highlightInTree(initial);
  }
})();
