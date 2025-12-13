/**
 * Bundle Size Benchmark
 * Compares the production bundle sizes of React vs Ferric todo apps
 */

import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';
import { gzipSizeSync } from 'gzip-size';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');

function getFileSize(filePath) {
  try {
    const stats = fs.statSync(filePath);
    return stats.size;
  } catch {
    return 0;
  }
}

function getGzipSize(filePath) {
  try {
    const content = fs.readFileSync(filePath);
    return gzipSizeSync(content);
  } catch {
    return 0;
  }
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

async function measureReactBundle() {
  const distDir = path.join(rootDir, 'react-todo', 'dist');
  const assetsDir = path.join(distDir, 'assets');

  let jsSize = 0;
  let jsGzip = 0;
  let cssSize = 0;
  let cssGzip = 0;

  if (await fs.pathExists(assetsDir)) {
    const files = await fs.readdir(assetsDir);

    for (const file of files) {
      const filePath = path.join(assetsDir, file);
      const size = getFileSize(filePath);
      const gzip = getGzipSize(filePath);

      if (file.endsWith('.js')) {
        jsSize += size;
        jsGzip += gzip;
      } else if (file.endsWith('.css')) {
        cssSize += size;
        cssGzip += gzip;
      }
    }
  }

  return {
    js: { raw: jsSize, gzip: jsGzip },
    css: { raw: cssSize, gzip: cssGzip },
    total: { raw: jsSize + cssSize, gzip: jsGzip + cssGzip },
  };
}

async function measureFerricBundle() {
  const distDir = path.join(rootDir, 'ferric-todo', 'dist');
  const pkgDir = path.join(distDir, 'pkg');

  let wasmSize = 0;
  let wasmGzip = 0;
  let jsSize = 0;
  let jsGzip = 0;
  let cssSize = 0;
  let cssGzip = 0;

  // WASM file
  const wasmFile = path.join(pkgDir, 'ferric_todo_benchmark_bg.wasm');
  if (await fs.pathExists(wasmFile)) {
    wasmSize = getFileSize(wasmFile);
    wasmGzip = getGzipSize(wasmFile);
  }

  // JS bindings
  const jsFile = path.join(pkgDir, 'ferric_todo_benchmark.js');
  if (await fs.pathExists(jsFile)) {
    jsSize = getFileSize(jsFile);
    jsGzip = getGzipSize(jsFile);
  }

  // CSS
  const cssFile = path.join(distDir, 'styles.css');
  if (await fs.pathExists(cssFile)) {
    cssSize = getFileSize(cssFile);
    cssGzip = getGzipSize(cssFile);
  }

  return {
    wasm: { raw: wasmSize, gzip: wasmGzip },
    js: { raw: jsSize, gzip: jsGzip },
    css: { raw: cssSize, gzip: cssGzip },
    total: { raw: wasmSize + jsSize + cssSize, gzip: wasmGzip + jsGzip + cssGzip },
  };
}

export async function runBundleSizeBenchmark() {
  console.log('Measuring bundle sizes...\n');

  const react = await measureReactBundle();
  const ferric = await measureFerricBundle();

  // Print comparison table
  console.log('┌─────────────────────────────────────────────────────────────┐');
  console.log('│                    Bundle Size Comparison                     │');
  console.log('├─────────────────────────────────────────────────────────────┤');
  console.log('│ Metric          │ React              │ Ferric             │');
  console.log('├─────────────────────────────────────────────────────────────┤');
  console.log(`│ JS (raw)        │ ${formatBytes(react.js.raw).padEnd(18)} │ ${formatBytes(ferric.js.raw).padEnd(18)} │`);
  console.log(`│ JS (gzip)       │ ${formatBytes(react.js.gzip).padEnd(18)} │ ${formatBytes(ferric.js.gzip).padEnd(18)} │`);
  console.log(`│ WASM (raw)      │ ${'N/A'.padEnd(18)} │ ${formatBytes(ferric.wasm.raw).padEnd(18)} │`);
  console.log(`│ WASM (gzip)     │ ${'N/A'.padEnd(18)} │ ${formatBytes(ferric.wasm.gzip).padEnd(18)} │`);
  console.log(`│ CSS (raw)       │ ${formatBytes(react.css.raw).padEnd(18)} │ ${formatBytes(ferric.css.raw).padEnd(18)} │`);
  console.log('├─────────────────────────────────────────────────────────────┤');
  console.log(`│ Total (raw)     │ ${formatBytes(react.total.raw).padEnd(18)} │ ${formatBytes(ferric.total.raw).padEnd(18)} │`);
  console.log(`│ Total (gzip)    │ ${formatBytes(react.total.gzip).padEnd(18)} │ ${formatBytes(ferric.total.gzip).padEnd(18)} │`);
  console.log('└─────────────────────────────────────────────────────────────┘');

  // Calculate winner
  const winner = react.total.gzip < ferric.total.gzip ? 'React' : 'Ferric';
  const diff = Math.abs(react.total.gzip - ferric.total.gzip);
  const pct = ((diff / Math.max(react.total.gzip, ferric.total.gzip)) * 100).toFixed(1);

  console.log(`\n🏆 ${winner} has ${pct}% smaller gzipped bundle`);

  return { react, ferric };
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runBundleSizeBenchmark();
}


 * Bundle Size Benchmark
 * Compares the production bundle sizes of React vs Ferric todo apps
 */

import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';
import { gzipSizeSync } from 'gzip-size';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');

function getFileSize(filePath) {
  try {
    const stats = fs.statSync(filePath);
    return stats.size;
  } catch {
    return 0;
  }
}

function getGzipSize(filePath) {
  try {
    const content = fs.readFileSync(filePath);
    return gzipSizeSync(content);
  } catch {
    return 0;
  }
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

async function measureReactBundle() {
  const distDir = path.join(rootDir, 'react-todo', 'dist');
  const assetsDir = path.join(distDir, 'assets');

  let jsSize = 0;
  let jsGzip = 0;
  let cssSize = 0;
  let cssGzip = 0;

  if (await fs.pathExists(assetsDir)) {
    const files = await fs.readdir(assetsDir);

    for (const file of files) {
      const filePath = path.join(assetsDir, file);
      const size = getFileSize(filePath);
      const gzip = getGzipSize(filePath);

      if (file.endsWith('.js')) {
        jsSize += size;
        jsGzip += gzip;
      } else if (file.endsWith('.css')) {
        cssSize += size;
        cssGzip += gzip;
      }
    }
  }

  return {
    js: { raw: jsSize, gzip: jsGzip },
    css: { raw: cssSize, gzip: cssGzip },
    total: { raw: jsSize + cssSize, gzip: jsGzip + cssGzip },
  };
}

async function measureFerricBundle() {
  const distDir = path.join(rootDir, 'ferric-todo', 'dist');
  const pkgDir = path.join(distDir, 'pkg');

  let wasmSize = 0;
  let wasmGzip = 0;
  let jsSize = 0;
  let jsGzip = 0;
  let cssSize = 0;
  let cssGzip = 0;

  // WASM file
  const wasmFile = path.join(pkgDir, 'ferric_todo_benchmark_bg.wasm');
  if (await fs.pathExists(wasmFile)) {
    wasmSize = getFileSize(wasmFile);
    wasmGzip = getGzipSize(wasmFile);
  }

  // JS bindings
  const jsFile = path.join(pkgDir, 'ferric_todo_benchmark.js');
  if (await fs.pathExists(jsFile)) {
    jsSize = getFileSize(jsFile);
    jsGzip = getGzipSize(jsFile);
  }

  // CSS
  const cssFile = path.join(distDir, 'styles.css');
  if (await fs.pathExists(cssFile)) {
    cssSize = getFileSize(cssFile);
    cssGzip = getGzipSize(cssFile);
  }

  return {
    wasm: { raw: wasmSize, gzip: wasmGzip },
    js: { raw: jsSize, gzip: jsGzip },
    css: { raw: cssSize, gzip: cssGzip },
    total: { raw: wasmSize + jsSize + cssSize, gzip: wasmGzip + jsGzip + cssGzip },
  };
}

export async function runBundleSizeBenchmark() {
  console.log('Measuring bundle sizes...\n');

  const react = await measureReactBundle();
  const ferric = await measureFerricBundle();

  // Print comparison table
  console.log('┌─────────────────────────────────────────────────────────────┐');
  console.log('│                    Bundle Size Comparison                     │');
  console.log('├─────────────────────────────────────────────────────────────┤');
  console.log('│ Metric          │ React              │ Ferric             │');
  console.log('├─────────────────────────────────────────────────────────────┤');
  console.log(`│ JS (raw)        │ ${formatBytes(react.js.raw).padEnd(18)} │ ${formatBytes(ferric.js.raw).padEnd(18)} │`);
  console.log(`│ JS (gzip)       │ ${formatBytes(react.js.gzip).padEnd(18)} │ ${formatBytes(ferric.js.gzip).padEnd(18)} │`);
  console.log(`│ WASM (raw)      │ ${'N/A'.padEnd(18)} │ ${formatBytes(ferric.wasm.raw).padEnd(18)} │`);
  console.log(`│ WASM (gzip)     │ ${'N/A'.padEnd(18)} │ ${formatBytes(ferric.wasm.gzip).padEnd(18)} │`);
  console.log(`│ CSS (raw)       │ ${formatBytes(react.css.raw).padEnd(18)} │ ${formatBytes(ferric.css.raw).padEnd(18)} │`);
  console.log('├─────────────────────────────────────────────────────────────┤');
  console.log(`│ Total (raw)     │ ${formatBytes(react.total.raw).padEnd(18)} │ ${formatBytes(ferric.total.raw).padEnd(18)} │`);
  console.log(`│ Total (gzip)    │ ${formatBytes(react.total.gzip).padEnd(18)} │ ${formatBytes(ferric.total.gzip).padEnd(18)} │`);
  console.log('└─────────────────────────────────────────────────────────────┘');

  // Calculate winner
  const winner = react.total.gzip < ferric.total.gzip ? 'React' : 'Ferric';
  const diff = Math.abs(react.total.gzip - ferric.total.gzip);
  const pct = ((diff / Math.max(react.total.gzip, ferric.total.gzip)) * 100).toFixed(1);

  console.log(`\n🏆 ${winner} has ${pct}% smaller gzipped bundle`);

  return { react, ferric };
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runBundleSizeBenchmark();
}

