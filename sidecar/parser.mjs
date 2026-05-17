// Mydoo Viewer Node sidecar — hwp / hwpx parsing
//
// Usage: node parser.mjs <file-path>
// Output (stdout): JSON { kind: "html", content: "<html>...</html>", text: "..." }
// Errors → stderr + exit 1

import { readFile, writeFile, unlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, extname } from "node:path";
import { randomBytes } from "node:crypto";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

async function parseHwpx(filePath) {
  const { HwpxReader } = await import("@ssabrojs/hwpxjs");
  const buffer = await readFile(filePath);
  const arrayBuffer = new Uint8Array(buffer).buffer;

  const reader = new HwpxReader();
  await reader.loadFromArrayBuffer(arrayBuffer);

  const text = (await reader.extractText().catch(() => "")) || "";
  let html = "";
  try {
    html = (await reader.extractHtml()) || "";
  } catch {
    html = "";
  }

  return { kind: "html", html, text };
}

async function parseHwp(filePath) {
  // node-hwp는 임시 파일 경로 필요. 원본 경로 그대로 넘긴다.
  const nodeHwp = require("node-hwp");
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

    // html이 비어 있고 text가 있으면 fallback html 생성
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
