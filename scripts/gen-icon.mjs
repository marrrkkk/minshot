// Generates src-tauri/icons/source.png (512x512) without dependencies.
// Dark rounded square + light-blue selection-rectangle glyph.
import { deflateSync } from "node:zlib";
import { writeFileSync, mkdirSync } from "node:fs";

const W = 512, H = 512;

function crc32(buf) {
  let table = crc32.t;
  if (!table) {
    table = crc32.t = new Int32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
      table[n] = c;
    }
  }
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = table[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

const px = Buffer.alloc(W * H * 4);
// background: near-black with subtle vertical gradient
for (let y = 0; y < H; y++) {
  for (let x = 0; x < W; x++) {
    const i = (y * W + x) * 4;
    const t = y / H;
    px[i] = Math.round(22 + 10 * t);
    px[i + 1] = Math.round(22 + 10 * t);
    px[i + 2] = Math.round(30 + 12 * t);
    px[i + 3] = 255;
  }
}
// rounded corners (transparent)
const R = 112;
const corner = (cx, cy) => {
  for (let y = 0; y < R; y++)
    for (let x = 0; x < R; x++) {
      const dx = R - x, dy = R - y;
      if (dx * dx + dy * dy > R * R) {
        const i = ((cy + y) * W + (cx + x)) * 4;
        px[i + 3] = 0;
      }
    }
};
corner(0, 0); corner(W - R, 0); corner(0, H - R); corner(W - R, H - R);

// selection rectangle glyph: light-blue stroke + handles
const x0 = 118, y0 = 150, x1 = 394, y1 = 342, TH = 22;
const BLUE = [77, 163, 255];
function rect(xa, ya, xb, yb, color, alpha = 255) {
  for (let y = ya; y < yb; y++)
    for (let x = xa; x < xb; x++) {
      if (x < 0 || y < 0 || x >= W || y >= H) continue;
      const i = (y * W + x) * 4;
      if (px[i + 3] === 0) continue;
      px[i] = color[0]; px[i + 1] = color[1]; px[i + 2] = color[2]; px[i + 3] = alpha;
    }
}
rect(x0, y0, x1, y0 + TH, BLUE);           // top
rect(x0, y1 - TH, x1, y1, BLUE);           // bottom
rect(x0, y0, x0 + TH, y1, BLUE);           // left
rect(x1 - TH, y0, x1, y1, BLUE);           // right
// faint fill
for (let y = y0 + TH; y < y1 - TH; y++)
  for (let x = x0 + TH; x < x1 - TH; x++) {
    const i = (y * W + x) * 4;
    px[i] = 77; px[i + 1] = 163; px[i + 2] = 255; px[i + 3] = 36;
  }
// corner handles
const HS = 44;
rect(x0 - HS / 2, y0 - HS / 2, x0 + HS / 2, y0 + HS / 2, [214, 235, 255]);
rect(x1 - HS / 2, y0 - HS / 2, x1 + HS / 2, y0 + HS / 2, [214, 235, 255]);
rect(x0 - HS / 2, y1 - HS / 2, x0 + HS / 2, y1 + HS / 2, [214, 235, 255]);
rect(x1 - HS / 2, y1 - HS / 2, x1 + HS / 2, y1 + HS / 2, [214, 235, 255]);

const raw = Buffer.alloc((W * 4 + 1) * H);
for (let y = 0; y < H; y++) {
  raw[y * (W * 4 + 1)] = 0;
  px.copy(raw, y * (W * 4 + 1) + 1, y * W * 4, (y + 1) * W * 4);
}
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(W, 0); ihdr.writeUInt32BE(H, 4);
ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", deflateSync(raw)),
  chunk("IEND", Buffer.alloc(0)),
]);
mkdirSync("src-tauri/icons", { recursive: true });
writeFileSync("src-tauri/icons/source.png", png);
console.log("wrote src-tauri/icons/source.png", png.length, "bytes");
