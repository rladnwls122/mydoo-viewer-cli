// 마이두 뷰어 — frontend logic
// Tauri command bridge via __TAURI__.core.invoke

const invoke = window.__TAURI__.core.invoke;

const $ = (sel) => document.querySelector(sel);
const tree = $("#tree");
const viewer = $("#viewer");
const currentFileLabel = $("#current-file");
const sidebarPath = $("#sidebar-path");

let currentDir = null;
let currentFile = null;

marked.setOptions({
  gfm: true,
  breaks: false,
  headerIds: true,
  mangle: false,
});

const FILE_ICON = {
  md: "📝", markdown: "📝",
  html: "🌐", htm: "🌐",
  txt: "📃",
  docx: "📘",
  hwp: "📕", hwpx: "📕",
};
const folderIcon = "📁";

function iconFor(name, isDir) {
  if (isDir) return folderIcon;
  const ext = name.split(".").pop().toLowerCase();
  return FILE_ICON[ext] || "📄";
}

async function loadTree(dir) {
  try {
    const entries = await invoke("list_dir", { path: dir });
    currentDir = dir;
    sidebarPath.textContent = dir;
    sidebarPath.title = dir;
    tree.innerHTML = "";
    for (const e of entries) {
      const li = document.createElement("li");
      li.dataset.path = e.path;
      li.dataset.isDir = e.is_dir;
      const icon = document.createElement("span");
      icon.className = "icon";
      icon.textContent = iconFor(e.name, e.is_dir);
      const label = document.createElement("span");
      label.className = "name";
      label.textContent = e.name;
      li.appendChild(icon);
      li.appendChild(label);
      li.addEventListener("click", () => {
        if (e.is_dir) {
          loadTree(e.path);
        } else {
          openFile(e.path);
          markActive(li);
        }
      });
      tree.appendChild(li);
    }
  } catch (err) {
    showError(`폴더를 읽을 수 없음: ${err}`);
  }
}

function markActive(li) {
  for (const el of tree.querySelectorAll("li.active")) el.classList.remove("active");
  li.classList.add("active");
}

async function openFile(path) {
  try {
    currentFile = path;
    currentFileLabel.textContent = path;
    document.title = `${baseName(path)} — 마이두 뷰어`;
    const file = await invoke("read_file", { path });
    renderFile(file);
  } catch (err) {
    showError(`파일을 열 수 없음: ${err}`);
  }
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
    const sanitized = DOMPurify.sanitize(content, {
      ADD_ATTR: ["target"],
      FORBID_TAGS: ["script", "iframe", "object", "embed", "link"],
      FORBID_ATTR: ["onerror", "onload", "onclick"],
    });
    viewer.innerHTML = sanitized;
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
    }).catch((e) => {
      showError(`docx 변환 실패: ${e}`);
    });
    return;
  }

  if (kind === "hwp" || kind === "hwpx") {
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
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

// 버튼들
$("#btn-open-folder").addEventListener("click", async () => {
  const path = await invoke("open_dir_dialog");
  if (path) loadTree(path);
});

$("#btn-open-file").addEventListener("click", async () => {
  const path = await invoke("open_file_dialog");
  if (path) {
    const parent = await invoke("parent_dir", { path });
    if (parent) await loadTree(parent);
    await openFile(path);
    highlightInTree(path);
  }
});

$("#btn-up").addEventListener("click", async () => {
  if (!currentDir) return;
  const parent = await invoke("parent_dir", { path: currentDir });
  if (parent && parent !== currentDir) loadTree(parent);
});

$("#btn-toggle-sidebar").addEventListener("click", () => {
  document.body.classList.toggle("no-sidebar");
});

function highlightInTree(path) {
  for (const li of tree.querySelectorAll("li")) {
    if (li.dataset.path === path) {
      markActive(li);
      li.scrollIntoView({ block: "nearest" });
      break;
    }
  }
}

// 키 바인딩
document.addEventListener("keydown", (e) => {
  const ctrl = e.ctrlKey || e.metaKey;
  if (!ctrl) return;
  if (e.key.toLowerCase() === "b") {
    e.preventDefault();
    document.body.classList.toggle("no-sidebar");
  } else if (e.key.toLowerCase() === "o") {
    e.preventDefault();
    $("#btn-open-file").click();
  } else if (e.key.toLowerCase() === "k") {
    e.preventDefault();
    $("#btn-open-folder").click();
  }
});

// 시작 시: 더블클릭으로 들어온 파일이 있으면 그걸 연다
(async () => {
  const initial = await invoke("initial_file");
  if (initial) {
    const parent = await invoke("parent_dir", { path: initial });
    if (parent) await loadTree(parent);
    await openFile(initial);
    highlightInTree(initial);
  }
})();
