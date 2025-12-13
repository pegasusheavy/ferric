/**
 * Memory Usage Benchmark
 * Measures heap usage at various states
 */

import puppeteer from 'puppeteer';
import { fileURLToPath } from 'url';

function formatMemory(bytes) {
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

async function getMemoryUsage(page) {
  const metrics = await page.metrics();
  return {
    jsHeapUsed: metrics.JSHeapUsedSize,
    jsHeapTotal: metrics.JSHeapTotalSize,
  };
}

async function measureAppMemory(browser, appUrl, appName) {
  console.log(`\nMeasuring ${appName} memory usage...`);
  const results = {};

  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 720 });

  // Initial load
  await page.goto(appUrl, { waitUntil: 'networkidle0' });
  await page.waitForSelector('.todo-list');

  // Force GC if available
  await page.evaluate(() => {
    if (window.gc) window.gc();
  });
  await page.waitForTimeout(100);

  results.initial = await getMemoryUsage(page);
  console.log(`  Initial: ${formatMemory(results.initial.jsHeapUsed)}`);

  // Add 100 todos
  await page.evaluate(() => window.__APP__.bulkAdd(100));
  await page.waitForTimeout(100);
  results.with100 = await getMemoryUsage(page);
  console.log(`  With 100 todos: ${formatMemory(results.with100.jsHeapUsed)}`);

  // Add 1000 more todos
  await page.evaluate(() => window.__APP__.bulkAdd(900));
  await page.waitForTimeout(100);
  results.with1000 = await getMemoryUsage(page);
  console.log(`  With 1000 todos: ${formatMemory(results.with1000.jsHeapUsed)}`);

  // Toggle all
  await page.evaluate(() => window.__APP__.toggleAll());
  await page.waitForTimeout(100);
  results.afterToggle = await getMemoryUsage(page);
  console.log(`  After toggle: ${formatMemory(results.afterToggle.jsHeapUsed)}`);

  // Clear completed
  await page.evaluate(() => window.__APP__.clearCompleted());
  await page.waitForTimeout(100);

  // Force GC
  await page.evaluate(() => {
    if (window.gc) window.gc();
  });
  await page.waitForTimeout(200);

  results.afterClear = await getMemoryUsage(page);
  console.log(`  After clear: ${formatMemory(results.afterClear.jsHeapUsed)}`);

  // Calculate memory per todo
  const mem100 = results.with100.jsHeapUsed - results.initial.jsHeapUsed;
  const mem1000 = results.with1000.jsHeapUsed - results.with100.jsHeapUsed;
  results.perTodo = {
    first100: mem100 / 100,
    next900: mem1000 / 900,
  };
  console.log(`  Memory per todo (first 100): ${(results.perTodo.first100 / 1024).toFixed(2)} KB`);
  console.log(`  Memory per todo (next 900): ${(results.perTodo.next900 / 1024).toFixed(2)} KB`);

  await page.close();
  return results;
}

export async function runMemoryBenchmark() {
  console.log('Starting memory benchmarks...');
  console.log('(Make sure both apps are running on their respective ports)\n');

  const browser = await puppeteer.launch({
    headless: true,
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--js-flags=--expose-gc',
    ],
  });

  try {
    const react = await measureAppMemory(browser, 'http://localhost:3000', 'React');
    const ferric = await measureAppMemory(browser, 'http://localhost:3001', 'Ferric');

    // Print comparison
    console.log('\n' + '='.repeat(70));
    console.log('Memory Usage Comparison (lower is better)');
    console.log('='.repeat(70));

    const metrics = [
      ['Initial Load', 'initial'],
      ['With 100 Todos', 'with100'],
      ['With 1000 Todos', 'with1000'],
      ['After Toggle All', 'afterToggle'],
      ['After Clear', 'afterClear'],
    ];

    console.log('┌────────────────────┬────────────────┬────────────────┬──────────┐');
    console.log('│ State              │ React          │ Ferric         │ Winner   │');
    console.log('├────────────────────┼────────────────┼────────────────┼──────────┤');

    for (const [name, key] of metrics) {
      const reactVal = react[key]?.jsHeapUsed;
      const ferricVal = ferric[key]?.jsHeapUsed;

      const reactStr = reactVal ? formatMemory(reactVal) : 'N/A';
      const ferricStr = ferricVal ? formatMemory(ferricVal) : 'N/A';

      let winner = '';
      if (reactVal && ferricVal) {
        winner = reactVal < ferricVal ? '✓ React' : '✓ Ferric';
      }

      console.log(`│ ${name.padEnd(18)} │ ${reactStr.padEnd(14)} │ ${ferricStr.padEnd(14)} │ ${winner.padEnd(8)} │`);
    }

    console.log('└────────────────────┴────────────────┴────────────────┴──────────┘');

    // Per-todo memory
    console.log('\nMemory per Todo Item:');
    console.log(`  React:  ${(react.perTodo.next900 / 1024).toFixed(2)} KB/todo`);
    console.log(`  Ferric: ${(ferric.perTodo.next900 / 1024).toFixed(2)} KB/todo`);

    const winner = react.perTodo.next900 < ferric.perTodo.next900 ? 'React' : 'Ferric';
    console.log(`  🏆 ${winner} uses less memory per todo`);

    return { react, ferric };
  } finally {
    await browser.close();
  }
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runMemoryBenchmark().catch(console.error);
}


 * Memory Usage Benchmark
 * Measures heap usage at various states
 */

import puppeteer from 'puppeteer';
import { fileURLToPath } from 'url';

function formatMemory(bytes) {
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

async function getMemoryUsage(page) {
  const metrics = await page.metrics();
  return {
    jsHeapUsed: metrics.JSHeapUsedSize,
    jsHeapTotal: metrics.JSHeapTotalSize,
  };
}

async function measureAppMemory(browser, appUrl, appName) {
  console.log(`\nMeasuring ${appName} memory usage...`);
  const results = {};

  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 720 });

  // Initial load
  await page.goto(appUrl, { waitUntil: 'networkidle0' });
  await page.waitForSelector('.todo-list');

  // Force GC if available
  await page.evaluate(() => {
    if (window.gc) window.gc();
  });
  await page.waitForTimeout(100);

  results.initial = await getMemoryUsage(page);
  console.log(`  Initial: ${formatMemory(results.initial.jsHeapUsed)}`);

  // Add 100 todos
  await page.evaluate(() => window.__APP__.bulkAdd(100));
  await page.waitForTimeout(100);
  results.with100 = await getMemoryUsage(page);
  console.log(`  With 100 todos: ${formatMemory(results.with100.jsHeapUsed)}`);

  // Add 1000 more todos
  await page.evaluate(() => window.__APP__.bulkAdd(900));
  await page.waitForTimeout(100);
  results.with1000 = await getMemoryUsage(page);
  console.log(`  With 1000 todos: ${formatMemory(results.with1000.jsHeapUsed)}`);

  // Toggle all
  await page.evaluate(() => window.__APP__.toggleAll());
  await page.waitForTimeout(100);
  results.afterToggle = await getMemoryUsage(page);
  console.log(`  After toggle: ${formatMemory(results.afterToggle.jsHeapUsed)}`);

  // Clear completed
  await page.evaluate(() => window.__APP__.clearCompleted());
  await page.waitForTimeout(100);

  // Force GC
  await page.evaluate(() => {
    if (window.gc) window.gc();
  });
  await page.waitForTimeout(200);

  results.afterClear = await getMemoryUsage(page);
  console.log(`  After clear: ${formatMemory(results.afterClear.jsHeapUsed)}`);

  // Calculate memory per todo
  const mem100 = results.with100.jsHeapUsed - results.initial.jsHeapUsed;
  const mem1000 = results.with1000.jsHeapUsed - results.with100.jsHeapUsed;
  results.perTodo = {
    first100: mem100 / 100,
    next900: mem1000 / 900,
  };
  console.log(`  Memory per todo (first 100): ${(results.perTodo.first100 / 1024).toFixed(2)} KB`);
  console.log(`  Memory per todo (next 900): ${(results.perTodo.next900 / 1024).toFixed(2)} KB`);

  await page.close();
  return results;
}

export async function runMemoryBenchmark() {
  console.log('Starting memory benchmarks...');
  console.log('(Make sure both apps are running on their respective ports)\n');

  const browser = await puppeteer.launch({
    headless: true,
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--js-flags=--expose-gc',
    ],
  });

  try {
    const react = await measureAppMemory(browser, 'http://localhost:3000', 'React');
    const ferric = await measureAppMemory(browser, 'http://localhost:3001', 'Ferric');

    // Print comparison
    console.log('\n' + '='.repeat(70));
    console.log('Memory Usage Comparison (lower is better)');
    console.log('='.repeat(70));

    const metrics = [
      ['Initial Load', 'initial'],
      ['With 100 Todos', 'with100'],
      ['With 1000 Todos', 'with1000'],
      ['After Toggle All', 'afterToggle'],
      ['After Clear', 'afterClear'],
    ];

    console.log('┌────────────────────┬────────────────┬────────────────┬──────────┐');
    console.log('│ State              │ React          │ Ferric         │ Winner   │');
    console.log('├────────────────────┼────────────────┼────────────────┼──────────┤');

    for (const [name, key] of metrics) {
      const reactVal = react[key]?.jsHeapUsed;
      const ferricVal = ferric[key]?.jsHeapUsed;

      const reactStr = reactVal ? formatMemory(reactVal) : 'N/A';
      const ferricStr = ferricVal ? formatMemory(ferricVal) : 'N/A';

      let winner = '';
      if (reactVal && ferricVal) {
        winner = reactVal < ferricVal ? '✓ React' : '✓ Ferric';
      }

      console.log(`│ ${name.padEnd(18)} │ ${reactStr.padEnd(14)} │ ${ferricStr.padEnd(14)} │ ${winner.padEnd(8)} │`);
    }

    console.log('└────────────────────┴────────────────┴────────────────┴──────────┘');

    // Per-todo memory
    console.log('\nMemory per Todo Item:');
    console.log(`  React:  ${(react.perTodo.next900 / 1024).toFixed(2)} KB/todo`);
    console.log(`  Ferric: ${(ferric.perTodo.next900 / 1024).toFixed(2)} KB/todo`);

    const winner = react.perTodo.next900 < ferric.perTodo.next900 ? 'React' : 'Ferric';
    console.log(`  🏆 ${winner} uses less memory per todo`);

    return { react, ferric };
  } finally {
    await browser.close();
  }
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runMemoryBenchmark().catch(console.error);
}

