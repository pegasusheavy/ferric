/**
 * Main benchmark runner
 * Runs all benchmark suites and generates a combined report
 */

import { runBundleSizeBenchmark } from './bundle-size.js';
import { runRuntimeBenchmark } from './runtime-perf.js';
import { runMemoryBenchmark } from './memory-usage.js';
import { generateReport } from './generate-report.js';
import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const resultsDir = path.join(__dirname, '..', 'results');

async function main() {
  console.log('🚀 Starting Todo App Benchmark Suite\n');
  console.log('='.repeat(60));

  // Ensure results directory exists
  await fs.ensureDir(resultsDir);

  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const results = {
    timestamp,
    environment: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
    },
    bundleSize: null,
    runtime: null,
    memory: null,
  };

  try {
    // Run bundle size benchmark
    console.log('\n📦 Running Bundle Size Benchmark...\n');
    results.bundleSize = await runBundleSizeBenchmark();

    // Run runtime performance benchmark
    console.log('\n⚡ Running Runtime Performance Benchmark...\n');
    results.runtime = await runRuntimeBenchmark();

    // Run memory usage benchmark
    console.log('\n💾 Running Memory Usage Benchmark...\n');
    results.memory = await runMemoryBenchmark();

    // Save raw results
    const resultsFile = path.join(resultsDir, `benchmark-${timestamp}.json`);
    await fs.writeJson(resultsFile, results, { spaces: 2 });
    console.log(`\n📊 Raw results saved to: ${resultsFile}`);

    // Generate readable report
    console.log('\n📝 Generating Report...\n');
    await generateReport(results);

  } catch (error) {
    console.error('❌ Benchmark failed:', error);
    process.exit(1);
  }

  console.log('\n✅ Benchmark suite completed!');
}

main();


 * Main benchmark runner
 * Runs all benchmark suites and generates a combined report
 */

import { runBundleSizeBenchmark } from './bundle-size.js';
import { runRuntimeBenchmark } from './runtime-perf.js';
import { runMemoryBenchmark } from './memory-usage.js';
import { generateReport } from './generate-report.js';
import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const resultsDir = path.join(__dirname, '..', 'results');

async function main() {
  console.log('🚀 Starting Todo App Benchmark Suite\n');
  console.log('='.repeat(60));

  // Ensure results directory exists
  await fs.ensureDir(resultsDir);

  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const results = {
    timestamp,
    environment: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
    },
    bundleSize: null,
    runtime: null,
    memory: null,
  };

  try {
    // Run bundle size benchmark
    console.log('\n📦 Running Bundle Size Benchmark...\n');
    results.bundleSize = await runBundleSizeBenchmark();

    // Run runtime performance benchmark
    console.log('\n⚡ Running Runtime Performance Benchmark...\n');
    results.runtime = await runRuntimeBenchmark();

    // Run memory usage benchmark
    console.log('\n💾 Running Memory Usage Benchmark...\n');
    results.memory = await runMemoryBenchmark();

    // Save raw results
    const resultsFile = path.join(resultsDir, `benchmark-${timestamp}.json`);
    await fs.writeJson(resultsFile, results, { spaces: 2 });
    console.log(`\n📊 Raw results saved to: ${resultsFile}`);

    // Generate readable report
    console.log('\n📝 Generating Report...\n');
    await generateReport(results);

  } catch (error) {
    console.error('❌ Benchmark failed:', error);
    process.exit(1);
  }

  console.log('\n✅ Benchmark suite completed!');
}

main();

