import sharp from "sharp";
import { readFile } from "node:fs/promises";

const [, , svgPath, outPath, sizeArg] = process.argv;
if (!svgPath || !outPath) {
  console.error("usage: node svg2png.mjs <svg> <png> [size=1024]");
  process.exit(1);
}
const size = Number(sizeArg) || 1024;
const svg = await readFile(svgPath);
await sharp(svg, { density: 300 })
  .resize(size, size)
  .png()
  .toFile(outPath);
console.log(`wrote ${outPath} (${size}x${size})`);
