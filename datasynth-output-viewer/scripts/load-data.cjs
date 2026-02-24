#!/usr/bin/env node
/**
 * Copy output directory to public/data so the viewer can load it.
 * Usage: OUTPUT_DIR=../output node scripts/load-data.cjs
 *        or npm run load-data (defaults to ./output)
 */

const fs = require('fs');
const path = require('path');
const src = path.resolve(process.cwd(), process.env.OUTPUT_DIR || path.join(process.cwd(), 'output'));
const dest = path.resolve(process.cwd(), 'public/data');

if (!fs.existsSync(src)) {
  console.warn('Output dir not found at', src, '- run datasynth generate first or set OUTPUT_DIR');
  process.exit(0);
}

if (!fs.existsSync(path.join(process.cwd(), 'public'))) {
  fs.mkdirSync(path.join(process.cwd(), 'public'), { recursive: true });
}
if (fs.existsSync(dest)) {
  fs.rmSync(dest, { recursive: true });
}
fs.mkdirSync(dest, { recursive: true });

function copyRecursive(from, to) {
  const entries = fs.readdirSync(from, { withFileTypes: true });
  for (const e of entries) {
    const s = path.join(from, e.name);
    const t = path.join(to, e.name);
    if (e.isDirectory()) {
      fs.mkdirSync(t, { recursive: true });
      copyRecursive(s, t);
    } else {
      fs.copyFileSync(s, t);
    }
  }
}

copyRecursive(src, dest);
console.log('Data loaded from', src, '->', dest);
