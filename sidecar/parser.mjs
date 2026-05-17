// Mydoo Viewer Node sidecar — hwp / hwpx parsing
//
// Usage: node parser.mjs <file-path>
// Output (stdout): JSON { kind: "html", html: "...", text: "..." }
// Errors → stderr + exit 1

import { readFile } from "node:fs/promises";
import { extname } from "node:path";
// 정적 import — Bun compile이 의존 모듈을 단일 실행파일에 묶기 위해 필수
import { HwpxReader } from "@ssabrojs/hwpxjs";
import nodeHwp from "node-hwp";
import JSZip from "jszip";

const MIME_BY_EXT = {
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  png: "image/png",
  gif: "image/gif",
  bmp: "image/bmp",
  svg: "image/svg+xml",
  webp: "image/webp",
};

async function parseHwpx(filePath) {
  const buffer = await readFile(filePath);
  // Buffer pool의 underlying ArrayBuffer는 파일보다 커서 ZIP 디코더가 EOCD를
  // 잘못 찾음 → 정확한 크기의 ArrayBuffer로 복사한다.
  const arrayBuffer = new ArrayBuffer(buffer.byteLength);
  new Uint8Array(arrayBuffer).set(buffer);

  const reader = new HwpxReader();
  await reader.loadFromArrayBuffer(arrayBuffer);

  const text = (await reader.extractText().catch(() => "")) || "";
  let html = "";
  try {
    html = (await reader.extractHtml()) || "";
  } catch {
    html = "";
  }

  // BinData/*.{jpg,png,...} 이미지를 base64 data URI로 인라인 — 깨진 src 정상화
  if (html.includes("BinData/")) {
    try {
      const zip = await JSZip.loadAsync(arrayBuffer);
      const images = {};
      await Promise.all(
        Object.keys(zip.files)
          .filter((p) => p.startsWith("BinData/"))
          .map(async (p) => {
            const ext = (p.split(".").pop() || "").toLowerCase();
            const mime = MIME_BY_EXT[ext];
            if (!mime) return;
            const data = await zip.files[p].async("base64");
            images[p] = `data:${mime};base64,${data}`;
          }),
      );
      html = html.replace(
        /src="(BinData\/[^"]+)"/g,
        (_, p) => `src="${images[p] || p}"`,
      );
    } catch {
      // 이미지 추출 실패해도 텍스트 렌더는 계속
    }
  }

  return { kind: "html", html, text };
}

async function parseHwp(filePath) {
  const doc = await new Promise((resolve, reject) => {
    nodeHwp.open(filePath, { type: "hwp" }, (err, d) => {
      if (err) reject(err);
      else resolve(d);
    });
  });

  let text = "";
  try {
    if (doc.convertTo && nodeHwp.converter?.plainText) {
      text = doc.convertTo(nodeHwp.converter.plainText) || "";
    }
  } catch {
    text = "";
  }
  return { kind: "text", html: "", text };
}

function textToHtml(text) {
  const escaped = text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
  const paragraphs = escaped
    .split(/\n\n+/)
    .map((p) => `<p>${p.replace(/\n/g, "<br>")}</p>`)
    .join("\n");
  return `<div class="hwp-fallback">${paragraphs}</div>`;
}

async function main() {
  const filePath = process.argv[2];
  if (!filePath) {
    console.error("usage: node parser.mjs <file-path>");
    process.exit(1);
  }
  const ext = extname(filePath).toLowerCase();
  try {
    let result;
    if (ext === ".hwpx") {
      result = await parseHwpx(filePath);
    } else if (ext === ".hwp") {
      result = await parseHwp(filePath);
    } else {
      console.error(`unsupported extension: ${ext}`);
      process.exit(1);
    }
    if (!result.html && result.text) {
      result.html = textToHtml(result.text);
    }
    process.stdout.write(JSON.stringify(result));
  } catch (err) {
    console.error(err?.message || String(err));
    process.exit(1);
  }
}

main();
