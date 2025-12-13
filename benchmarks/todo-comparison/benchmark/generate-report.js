/**
 * Generate human-readable benchmark report
 */

import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const resultsDir = path.join(__dirname, '..', 'results');

function formatBytes(bytes) {
  if (!bytes) return 'N/A';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function formatMs(ms) {
  if (!ms) return 'N/A';
  return `${ms.toFixed(2)}ms`;
}

function calculateDiff(a, b) {
  if (!a || !b) return { diff: 0, pct: 0, winner: null };
  const diff = Math.abs(a - b);
  const pct = ((diff / Math.max(a, b)) * 100);
  const winner = a < b ? 'React' : 'Ferric';
  return { diff, pct, winner };
}

export async function generateReport(results) {
  const lines = [];

  lines.push('# Todo App Benchmark Report');
  lines.push('');
  lines.push(`**Generated:** ${new Date().toISOString()}`);
  lines.push('');
  lines.push(`**Environment:**`);
  lines.push(`- Node.js: ${results.environment.node}`);
  lines.push(`- Platform: ${results.environment.platform} (${results.environment.arch})`);
  lines.push('');

  // Bundle Size
  if (results.bundleSize) {
    lines.push('## 📦 Bundle Size');
    lines.push('');
    lines.push('| Metric | React | Ferric | Difference |');
    lines.push('|--------|-------|--------|------------|');

    const { react, ferric } = results.bundleSize;

    lines.push(`| JS (raw) | ${formatBytes(react.js.raw)} | ${formatBytes(ferric.js.raw)} | - |`);
    lines.push(`| JS (gzip) | ${formatBytes(react.js.gzip)} | ${formatBytes(ferric.js.gzip)} | - |`);
    lines.push(`| WASM (raw) | N/A | ${formatBytes(ferric.wasm?.raw)} | - |`);
    lines.push(`| WASM (gzip) | N/A | ${formatBytes(ferric.wasm?.gzip)} | - |`);
    lines.push(`| CSS (raw) | ${formatBytes(react.css.raw)} | ${formatBytes(ferric.css.raw)} | - |`);

    const totalDiff = calculateDiff(react.total.gzip, ferric.total.gzip);
    lines.push(`| **Total (gzip)** | **${formatBytes(react.total.gzip)}** | **${formatBytes(ferric.total.gzip)}** | **${totalDiff.winner} wins by ${totalDiff.pct.toFixed(1)}%** |`);
    lines.push('');
  }

  // Runtime Performance
  if (results.runtime) {
    lines.push('## ⚡ Runtime Performance');
    lines.push('');
    lines.push('*Lower is better*');
    lines.push('');
    lines.push('| Operation | React | Ferric | Winner |');
    lines.push('|-----------|-------|--------|--------|');

    const { react, ferric } = results.runtime;

    const ops = [
      ['Initial Load', react.initialLoad, ferric.initialLoad],
      ['Add 100 Todos', react.add100?.avg, ferric.add100?.avg],
      ['Add 1000 Todos', react.add1000?.avg, ferric.add1000?.avg],
      ['Toggle All', react.toggleAll?.avg, ferric.toggleAll?.avg],
      ['Filter Active', react.filterActive?.avg, ferric.filterActive?.avg],
      ['Filter Completed', react.filterCompleted?.avg, ferric.filterCompleted?.avg],
      ['Clear Completed', react.clearCompleted?.avg, ferric.clearCompleted?.avg],
    ];

    for (const [name, reactVal, ferricVal] of ops) {
      const diff = calculateDiff(reactVal, ferricVal);
      lines.push(`| ${name} | ${formatMs(reactVal)} | ${formatMs(ferricVal)} | ${diff.winner ? `✓ ${diff.winner}` : '-'} |`);
    }
    lines.push('');
  }

  // Memory Usage
  if (results.memory) {
    lines.push('## 💾 Memory Usage');
    lines.push('');
    lines.push('*Lower is better*');
    lines.push('');
    lines.push('| State | React | Ferric | Winner |');
    lines.push('|-------|-------|--------|--------|');

    const { react, ferric } = results.memory;

    const states = [
      ['Initial', react.initial?.jsHeapUsed, ferric.initial?.jsHeapUsed],
      ['100 Todos', react.with100?.jsHeapUsed, ferric.with100?.jsHeapUsed],
      ['1000 Todos', react.with1000?.jsHeapUsed, ferric.with1000?.jsHeapUsed],
      ['After Toggle', react.afterToggle?.jsHeapUsed, ferric.afterToggle?.jsHeapUsed],
      ['After Clear', react.afterClear?.jsHeapUsed, ferric.afterClear?.jsHeapUsed],
    ];

    for (const [name, reactVal, ferricVal] of states) {
      const diff = calculateDiff(reactVal, ferricVal);
      lines.push(`| ${name} | ${formatBytes(reactVal)} | ${formatBytes(ferricVal)} | ${diff.winner ? `✓ ${diff.winner}` : '-'} |`);
    }

    lines.push('');
    lines.push('### Memory per Todo');
    lines.push('');
    lines.push(`- React: ${((react.perTodo?.next900 || 0) / 1024).toFixed(2)} KB/todo`);
    lines.push(`- Ferric: ${((ferric.perTodo?.next900 || 0) / 1024).toFixed(2)} KB/todo`);
    lines.push('');
  }

  // Summary
  lines.push('## 📊 Summary');
  lines.push('');

  let reactWins = 0;
  let ferricWins = 0;

  // Count wins from bundle size
  if (results.bundleSize) {
    const { react, ferric } = results.bundleSize;
    if (react.total.gzip < ferric.total.gzip) reactWins++;
    else ferricWins++;
  }

  // Count wins from runtime
  if (results.runtime) {
    const { react, ferric } = results.runtime;
    if (react.initialLoad < ferric.initialLoad) reactWins++;
    else ferricWins++;
    if ((react.add1000?.avg || Infinity) < (ferric.add1000?.avg || Infinity)) reactWins++;
    else ferricWins++;
    if ((react.toggleAll?.avg || Infinity) < (ferric.toggleAll?.avg || Infinity)) reactWins++;
    else ferricWins++;
  }

  // Count wins from memory
  if (results.memory) {
    const { react, ferric } = results.memory;
    if ((react.with1000?.jsHeapUsed || Infinity) < (ferric.with1000?.jsHeapUsed || Infinity)) reactWins++;
    else ferricWins++;
  }

  lines.push(`| Framework | Wins |`);
  lines.push(`|-----------|------|`);
  lines.push(`| React | ${reactWins} |`);
  lines.push(`| Ferric | ${ferricWins} |`);
  lines.push('');

  const overallWinner = reactWins > ferricWins ? 'React' : (ferricWins > reactWins ? 'Ferric' : 'Tie');
  lines.push(`**Overall Winner: ${overallWinner}** 🏆`);
  lines.push('');

  lines.push('---');
  lines.push('');
  lines.push('*Note: Results may vary based on hardware, browser, and runtime conditions.*');

  const report = lines.join('\n');

  // Save report
  const reportFile = path.join(resultsDir, `report-${results.timestamp}.md`);
  await fs.writeFile(reportFile, report);
  console.log(`Report saved to: ${reportFile}`);

  // Also save as latest
  const latestFile = path.join(resultsDir, 'LATEST.md');
  await fs.writeFile(latestFile, report);

  // Print to console
  console.log('\n' + report);

  return report;
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  // Read latest results
  const files = await fs.readdir(resultsDir);
  const jsonFiles = files.filter(f => f.endsWith('.json')).sort().reverse();

  if (jsonFiles.length > 0) {
    const latest = await fs.readJson(path.join(resultsDir, jsonFiles[0]));
    await generateReport(latest);
  } else {
    console.log('No benchmark results found. Run the benchmark first.');
  }
}


 * Generate human-readable benchmark report
 */

import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const resultsDir = path.join(__dirname, '..', 'results');

function formatBytes(bytes) {
  if (!bytes) return 'N/A';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function formatMs(ms) {
  if (!ms) return 'N/A';
  return `${ms.toFixed(2)}ms`;
}

function calculateDiff(a, b) {
  if (!a || !b) return { diff: 0, pct: 0, winner: null };
  const diff = Math.abs(a - b);
  const pct = ((diff / Math.max(a, b)) * 100);
  const winner = a < b ? 'React' : 'Ferric';
  return { diff, pct, winner };
}

export async function generateReport(results) {
  const lines = [];

  lines.push('# Todo App Benchmark Report');
  lines.push('');
  lines.push(`**Generated:** ${new Date().toISOString()}`);
  lines.push('');
  lines.push(`**Environment:**`);
  lines.push(`- Node.js: ${results.environment.node}`);
  lines.push(`- Platform: ${results.environment.platform} (${results.environment.arch})`);
  lines.push('');

  // Bundle Size
  if (results.bundleSize) {
    lines.push('## 📦 Bundle Size');
    lines.push('');
    lines.push('| Metric | React | Ferric | Difference |');
    lines.push('|--------|-------|--------|------------|');

    const { react, ferric } = results.bundleSize;

    lines.push(`| JS (raw) | ${formatBytes(react.js.raw)} | ${formatBytes(ferric.js.raw)} | - |`);
    lines.push(`| JS (gzip) | ${formatBytes(react.js.gzip)} | ${formatBytes(ferric.js.gzip)} | - |`);
    lines.push(`| WASM (raw) | N/A | ${formatBytes(ferric.wasm?.raw)} | - |`);
    lines.push(`| WASM (gzip) | N/A | ${formatBytes(ferric.wasm?.gzip)} | - |`);
    lines.push(`| CSS (raw) | ${formatBytes(react.css.raw)} | ${formatBytes(ferric.css.raw)} | - |`);

    const totalDiff = calculateDiff(react.total.gzip, ferric.total.gzip);
    lines.push(`| **Total (gzip)** | **${formatBytes(react.total.gzip)}** | **${formatBytes(ferric.total.gzip)}** | **${totalDiff.winner} wins by ${totalDiff.pct.toFixed(1)}%** |`);
    lines.push('');
  }

  // Runtime Performance
  if (results.runtime) {
    lines.push('## ⚡ Runtime Performance');
    lines.push('');
    lines.push('*Lower is better*');
    lines.push('');
    lines.push('| Operation | React | Ferric | Winner |');
    lines.push('|-----------|-------|--------|--------|');

    const { react, ferric } = results.runtime;

    const ops = [
      ['Initial Load', react.initialLoad, ferric.initialLoad],
      ['Add 100 Todos', react.add100?.avg, ferric.add100?.avg],
      ['Add 1000 Todos', react.add1000?.avg, ferric.add1000?.avg],
      ['Toggle All', react.toggleAll?.avg, ferric.toggleAll?.avg],
      ['Filter Active', react.filterActive?.avg, ferric.filterActive?.avg],
      ['Filter Completed', react.filterCompleted?.avg, ferric.filterCompleted?.avg],
      ['Clear Completed', react.clearCompleted?.avg, ferric.clearCompleted?.avg],
    ];

    for (const [name, reactVal, ferricVal] of ops) {
      const diff = calculateDiff(reactVal, ferricVal);
      lines.push(`| ${name} | ${formatMs(reactVal)} | ${formatMs(ferricVal)} | ${diff.winner ? `✓ ${diff.winner}` : '-'} |`);
    }
    lines.push('');
  }

  // Memory Usage
  if (results.memory) {
    lines.push('## 💾 Memory Usage');
    lines.push('');
    lines.push('*Lower is better*');
    lines.push('');
    lines.push('| State | React | Ferric | Winner |');
    lines.push('|-------|-------|--------|--------|');

    const { react, ferric } = results.memory;

    const states = [
      ['Initial', react.initial?.jsHeapUsed, ferric.initial?.jsHeapUsed],
      ['100 Todos', react.with100?.jsHeapUsed, ferric.with100?.jsHeapUsed],
      ['1000 Todos', react.with1000?.jsHeapUsed, ferric.with1000?.jsHeapUsed],
      ['After Toggle', react.afterToggle?.jsHeapUsed, ferric.afterToggle?.jsHeapUsed],
      ['After Clear', react.afterClear?.jsHeapUsed, ferric.afterClear?.jsHeapUsed],
    ];

    for (const [name, reactVal, ferricVal] of states) {
      const diff = calculateDiff(reactVal, ferricVal);
      lines.push(`| ${name} | ${formatBytes(reactVal)} | ${formatBytes(ferricVal)} | ${diff.winner ? `✓ ${diff.winner}` : '-'} |`);
    }

    lines.push('');
    lines.push('### Memory per Todo');
    lines.push('');
    lines.push(`- React: ${((react.perTodo?.next900 || 0) / 1024).toFixed(2)} KB/todo`);
    lines.push(`- Ferric: ${((ferric.perTodo?.next900 || 0) / 1024).toFixed(2)} KB/todo`);
    lines.push('');
  }

  // Summary
  lines.push('## 📊 Summary');
  lines.push('');

  let reactWins = 0;
  let ferricWins = 0;

  // Count wins from bundle size
  if (results.bundleSize) {
    const { react, ferric } = results.bundleSize;
    if (react.total.gzip < ferric.total.gzip) reactWins++;
    else ferricWins++;
  }

  // Count wins from runtime
  if (results.runtime) {
    const { react, ferric } = results.runtime;
    if (react.initialLoad < ferric.initialLoad) reactWins++;
    else ferricWins++;
    if ((react.add1000?.avg || Infinity) < (ferric.add1000?.avg || Infinity)) reactWins++;
    else ferricWins++;
    if ((react.toggleAll?.avg || Infinity) < (ferric.toggleAll?.avg || Infinity)) reactWins++;
    else ferricWins++;
  }

  // Count wins from memory
  if (results.memory) {
    const { react, ferric } = results.memory;
    if ((react.with1000?.jsHeapUsed || Infinity) < (ferric.with1000?.jsHeapUsed || Infinity)) reactWins++;
    else ferricWins++;
  }

  lines.push(`| Framework | Wins |`);
  lines.push(`|-----------|------|`);
  lines.push(`| React | ${reactWins} |`);
  lines.push(`| Ferric | ${ferricWins} |`);
  lines.push('');

  const overallWinner = reactWins > ferricWins ? 'React' : (ferricWins > reactWins ? 'Ferric' : 'Tie');
  lines.push(`**Overall Winner: ${overallWinner}** 🏆`);
  lines.push('');

  lines.push('---');
  lines.push('');
  lines.push('*Note: Results may vary based on hardware, browser, and runtime conditions.*');

  const report = lines.join('\n');

  // Save report
  const reportFile = path.join(resultsDir, `report-${results.timestamp}.md`);
  await fs.writeFile(reportFile, report);
  console.log(`Report saved to: ${reportFile}`);

  // Also save as latest
  const latestFile = path.join(resultsDir, 'LATEST.md');
  await fs.writeFile(latestFile, report);

  // Print to console
  console.log('\n' + report);

  return report;
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  // Read latest results
  const files = await fs.readdir(resultsDir);
  const jsonFiles = files.filter(f => f.endsWith('.json')).sort().reverse();

  if (jsonFiles.length > 0) {
    const latest = await fs.readJson(path.join(resultsDir, jsonFiles[0]));
    await generateReport(latest);
  } else {
    console.log('No benchmark results found. Run the benchmark first.');
  }
}

