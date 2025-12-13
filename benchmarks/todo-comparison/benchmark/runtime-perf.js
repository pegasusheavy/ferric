/**
 * Runtime Performance Benchmark
 * Measures various operations using Puppeteer
 */

import puppeteer from 'puppeteer';
import { fileURLToPath } from 'url';

const ITERATIONS = 5;
const WARMUP = 2;

async function runBenchmark(page, name, fn) {
  const times = [];

  // Warmup
  for (let i = 0; i < WARMUP; i++) {
    await fn(page);
  }

  // Actual runs
  for (let i = 0; i < ITERATIONS; i++) {
    const start = await page.evaluate(() => performance.now());
    await fn(page);
    const end = await page.evaluate(() => performance.now());
    times.push(end - start);
  }

  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const min = Math.min(...times);
  const max = Math.max(...times);

  return { name, avg, min, max, times };
}

async function measureApp(browser, appUrl, appName) {
  console.log(`\nBenchmarking ${appName}...`);
  const results = {};

  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 720 });

  // Initial load
  const loadStart = Date.now();
  await page.goto(appUrl, { waitUntil: 'networkidle0' });
  results.initialLoad = Date.now() - loadStart;
  console.log(`  Initial load: ${results.initialLoad}ms`);

  // Wait for app to be ready
  await page.waitForSelector('.todo-list');

  // Add 100 todos
  results.add100 = await runBenchmark(page, 'Add 100 todos', async (p) => {
    await p.evaluate(() => window.__APP__.bulkAdd(100));
    await p.waitForFunction(() => {
      const list = document.querySelector('.todo-list');
      return list && list.children.length >= 100;
    });
  });
  console.log(`  Add 100: ${results.add100.avg.toFixed(2)}ms`);

  // Clear for next test
  await page.evaluate(() => window.__APP__.clearCompleted?.() || location.reload());
  await page.waitForTimeout(100);

  // Add 1000 todos
  results.add1000 = await runBenchmark(page, 'Add 1000 todos', async (p) => {
    await p.evaluate(() => window.__APP__.bulkAdd(1000));
    await p.waitForFunction(() => {
      const list = document.querySelector('.todo-list');
      return list && list.children.length >= 1000;
    });
  });
  console.log(`  Add 1000: ${results.add1000.avg.toFixed(2)}ms`);

  // Toggle all (with 1000 todos)
  results.toggleAll = await runBenchmark(page, 'Toggle all', async (p) => {
    await p.evaluate(() => window.__APP__.toggleAll());
  });
  console.log(`  Toggle all: ${results.toggleAll.avg.toFixed(2)}ms`);

  // Filter to active
  results.filterActive = await runBenchmark(page, 'Filter active', async (p) => {
    await p.evaluate(() => window.__APP__.setFilter('active'));
  });
  console.log(`  Filter active: ${results.filterActive.avg.toFixed(2)}ms`);

  // Filter to completed
  results.filterCompleted = await runBenchmark(page, 'Filter completed', async (p) => {
    await p.evaluate(() => window.__APP__.setFilter('completed'));
  });
  console.log(`  Filter completed: ${results.filterCompleted.avg.toFixed(2)}ms`);

  // Clear completed
  results.clearCompleted = await runBenchmark(page, 'Clear completed', async (p) => {
    await p.evaluate(() => window.__APP__.clearCompleted());
  });
  console.log(`  Clear completed: ${results.clearCompleted.avg.toFixed(2)}ms`);

  await page.close();
  return results;
}

export async function runRuntimeBenchmark() {
  console.log('Starting runtime benchmarks...');
  console.log('(Make sure both apps are running on their respective ports)\n');

  const browser = await puppeteer.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  try {
    const react = await measureApp(browser, 'http://localhost:3000', 'React');
    const ferric = await measureApp(browser, 'http://localhost:3001', 'Ferric');

    // Print comparison
    console.log('\n' + '='.repeat(70));
    console.log('Runtime Performance Comparison (average ms, lower is better)');
    console.log('='.repeat(70));

    const metrics = [
      ['Initial Load', 'initialLoad', true],
      ['Add 100 Todos', 'add100', false],
      ['Add 1000 Todos', 'add1000', false],
      ['Toggle All', 'toggleAll', false],
      ['Filter Active', 'filterActive', false],
      ['Filter Completed', 'filterCompleted', false],
      ['Clear Completed', 'clearCompleted', false],
    ];

    console.log('┌────────────────────┬────────────┬────────────┬──────────┐');
    console.log('│ Metric             │ React      │ Ferric     │ Winner   │');
    console.log('├────────────────────┼────────────┼────────────┼──────────┤');

    for (const [name, key, isSimple] of metrics) {
      const reactVal = isSimple ? react[key] : react[key]?.avg;
      const ferricVal = isSimple ? ferric[key] : ferric[key]?.avg;

      const reactStr = reactVal ? `${reactVal.toFixed(2)}ms` : 'N/A';
      const ferricStr = ferricVal ? `${ferricVal.toFixed(2)}ms` : 'N/A';

      let winner = '';
      if (reactVal && ferricVal) {
        winner = reactVal < ferricVal ? '✓ React' : '✓ Ferric';
      }

      console.log(`│ ${name.padEnd(18)} │ ${reactStr.padEnd(10)} │ ${ferricStr.padEnd(10)} │ ${winner.padEnd(8)} │`);
    }

    console.log('└────────────────────┴────────────┴────────────┴──────────┘');

    return { react, ferric };
  } finally {
    await browser.close();
  }
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runRuntimeBenchmark().catch(console.error);
}


 * Runtime Performance Benchmark
 * Measures various operations using Puppeteer
 */

import puppeteer from 'puppeteer';
import { fileURLToPath } from 'url';

const ITERATIONS = 5;
const WARMUP = 2;

async function runBenchmark(page, name, fn) {
  const times = [];

  // Warmup
  for (let i = 0; i < WARMUP; i++) {
    await fn(page);
  }

  // Actual runs
  for (let i = 0; i < ITERATIONS; i++) {
    const start = await page.evaluate(() => performance.now());
    await fn(page);
    const end = await page.evaluate(() => performance.now());
    times.push(end - start);
  }

  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const min = Math.min(...times);
  const max = Math.max(...times);

  return { name, avg, min, max, times };
}

async function measureApp(browser, appUrl, appName) {
  console.log(`\nBenchmarking ${appName}...`);
  const results = {};

  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 720 });

  // Initial load
  const loadStart = Date.now();
  await page.goto(appUrl, { waitUntil: 'networkidle0' });
  results.initialLoad = Date.now() - loadStart;
  console.log(`  Initial load: ${results.initialLoad}ms`);

  // Wait for app to be ready
  await page.waitForSelector('.todo-list');

  // Add 100 todos
  results.add100 = await runBenchmark(page, 'Add 100 todos', async (p) => {
    await p.evaluate(() => window.__APP__.bulkAdd(100));
    await p.waitForFunction(() => {
      const list = document.querySelector('.todo-list');
      return list && list.children.length >= 100;
    });
  });
  console.log(`  Add 100: ${results.add100.avg.toFixed(2)}ms`);

  // Clear for next test
  await page.evaluate(() => window.__APP__.clearCompleted?.() || location.reload());
  await page.waitForTimeout(100);

  // Add 1000 todos
  results.add1000 = await runBenchmark(page, 'Add 1000 todos', async (p) => {
    await p.evaluate(() => window.__APP__.bulkAdd(1000));
    await p.waitForFunction(() => {
      const list = document.querySelector('.todo-list');
      return list && list.children.length >= 1000;
    });
  });
  console.log(`  Add 1000: ${results.add1000.avg.toFixed(2)}ms`);

  // Toggle all (with 1000 todos)
  results.toggleAll = await runBenchmark(page, 'Toggle all', async (p) => {
    await p.evaluate(() => window.__APP__.toggleAll());
  });
  console.log(`  Toggle all: ${results.toggleAll.avg.toFixed(2)}ms`);

  // Filter to active
  results.filterActive = await runBenchmark(page, 'Filter active', async (p) => {
    await p.evaluate(() => window.__APP__.setFilter('active'));
  });
  console.log(`  Filter active: ${results.filterActive.avg.toFixed(2)}ms`);

  // Filter to completed
  results.filterCompleted = await runBenchmark(page, 'Filter completed', async (p) => {
    await p.evaluate(() => window.__APP__.setFilter('completed'));
  });
  console.log(`  Filter completed: ${results.filterCompleted.avg.toFixed(2)}ms`);

  // Clear completed
  results.clearCompleted = await runBenchmark(page, 'Clear completed', async (p) => {
    await p.evaluate(() => window.__APP__.clearCompleted());
  });
  console.log(`  Clear completed: ${results.clearCompleted.avg.toFixed(2)}ms`);

  await page.close();
  return results;
}

export async function runRuntimeBenchmark() {
  console.log('Starting runtime benchmarks...');
  console.log('(Make sure both apps are running on their respective ports)\n');

  const browser = await puppeteer.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  try {
    const react = await measureApp(browser, 'http://localhost:3000', 'React');
    const ferric = await measureApp(browser, 'http://localhost:3001', 'Ferric');

    // Print comparison
    console.log('\n' + '='.repeat(70));
    console.log('Runtime Performance Comparison (average ms, lower is better)');
    console.log('='.repeat(70));

    const metrics = [
      ['Initial Load', 'initialLoad', true],
      ['Add 100 Todos', 'add100', false],
      ['Add 1000 Todos', 'add1000', false],
      ['Toggle All', 'toggleAll', false],
      ['Filter Active', 'filterActive', false],
      ['Filter Completed', 'filterCompleted', false],
      ['Clear Completed', 'clearCompleted', false],
    ];

    console.log('┌────────────────────┬────────────┬────────────┬──────────┐');
    console.log('│ Metric             │ React      │ Ferric     │ Winner   │');
    console.log('├────────────────────┼────────────┼────────────┼──────────┤');

    for (const [name, key, isSimple] of metrics) {
      const reactVal = isSimple ? react[key] : react[key]?.avg;
      const ferricVal = isSimple ? ferric[key] : ferric[key]?.avg;

      const reactStr = reactVal ? `${reactVal.toFixed(2)}ms` : 'N/A';
      const ferricStr = ferricVal ? `${ferricVal.toFixed(2)}ms` : 'N/A';

      let winner = '';
      if (reactVal && ferricVal) {
        winner = reactVal < ferricVal ? '✓ React' : '✓ Ferric';
      }

      console.log(`│ ${name.padEnd(18)} │ ${reactStr.padEnd(10)} │ ${ferricStr.padEnd(10)} │ ${winner.padEnd(8)} │`);
    }

    console.log('└────────────────────┴────────────┴────────────┴──────────┘');

    return { react, ferric };
  } finally {
    await browser.close();
  }
}

// Run directly if called as script
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runRuntimeBenchmark().catch(console.error);
}

