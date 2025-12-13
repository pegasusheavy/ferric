/**
 * Build command implementation
 */

import path from 'node:path';
import fs from 'fs-extra';
import { glob } from 'glob';
import pc from 'picocolors';
import { loadConfig, type FerricConfig } from '../config/loader.js';
import { compileTypeScript, bundleFiles } from '../compiler/swc.js';
import { generateHtmlFromConfig, writeHtml } from '../html/generator.js';
import { watch as chokidarWatch } from 'chokidar';

export interface BuildOptions {
  outdir?: string;
  watch?: boolean;
  minify?: boolean;
  sourcemap?: boolean;
  config?: string;
}

/**
 * Build a Ferric project
 */
export async function build(options: BuildOptions): Promise<void> {
  const startTime = Date.now();

  console.log(pc.cyan('\n🔧 Ferric Build\n'));

  // Load configuration
  const config = await loadConfig(options.config);

  // Override config with CLI options
  if (options.outdir) config.outDir = options.outdir;
  if (options.minify === false) config.build = { ...config.build, minify: false };
  if (options.sourcemap === false) config.build = { ...config.build, sourcemap: false };

  const outDir = config.outDir || 'dist';

  // Clean output directory
  console.log(pc.gray(`Cleaning ${outDir}...`));
  await fs.emptyDir(outDir);

  // Build steps
  await Promise.all([
    buildWasm(config, outDir),
    buildTypeScript(config, outDir),
    buildAssets(config, outDir),
    buildHtml(config, outDir),
  ]);

  const elapsed = Date.now() - startTime;
  console.log(pc.green(`\n✓ Build completed in ${elapsed}ms\n`));

  // Watch mode
  if (options.watch) {
    console.log(pc.cyan('Watching for changes...\n'));
    await watchForChanges(config, outDir);
  }
}

/**
 * Copy WASM package files
 */
async function buildWasm(config: FerricConfig, outDir: string): Promise<void> {
  const pkgDir = config.wasm?.pkgDir || './pkg';
  const moduleName = config.wasm?.moduleName || 'ferric';

  console.log(pc.gray(`Copying WASM files from ${pkgDir}...`));

  if (!await fs.pathExists(pkgDir)) {
    console.log(pc.yellow(`  ⚠ WASM package not found at ${pkgDir}`));
    console.log(pc.yellow(`    Run 'wasm-pack build --target web' first`));
    return;
  }

  // Copy WASM files
  const wasmFiles = [
    `${moduleName}.js`,
    `${moduleName}_bg.wasm`,
    `${moduleName}_bg.wasm.d.ts`,
    `${moduleName}.d.ts`,
  ];

  for (const file of wasmFiles) {
    const src = path.join(pkgDir, file);
    const dest = path.join(outDir, file);

    if (await fs.pathExists(src)) {
      await fs.copy(src, dest);
      console.log(pc.gray(`  → ${file}`));
    }
  }
}

/**
 * Compile TypeScript source files
 */
async function buildTypeScript(config: FerricConfig, outDir: string): Promise<void> {
  const entry = config.entry || './src/main.ts';

  console.log(pc.gray(`Compiling TypeScript...`));

  // Check if entry exists
  if (!await fs.pathExists(entry)) {
    console.log(pc.yellow(`  ⚠ Entry file not found at ${entry}`));
    return;
  }

  // Find all TypeScript files
  const srcDir = path.dirname(entry);
  const files = await glob(`${srcDir}/**/*.{ts,tsx}`, {
    ignore: ['**/*.d.ts', '**/node_modules/**'],
  });

  if (files.length === 0) {
    console.log(pc.yellow(`  ⚠ No TypeScript files found`));
    return;
  }

  await compileTypeScript({
    input: files,
    outDir: path.join(outDir, 'js'),
    minify: config.build?.minify ?? true,
    sourcemap: config.build?.sourcemap ?? true,
    target: config.build?.target || 'es2022',
  });

  console.log(pc.gray(`  → Compiled ${files.length} files`));
}

/**
 * Copy static assets
 */
async function buildAssets(config: FerricConfig, outDir: string): Promise<void> {
  const assetDirs = config.assets || ['public'];

  console.log(pc.gray(`Copying assets...`));

  for (const assetDir of assetDirs) {
    if (await fs.pathExists(assetDir)) {
      await fs.copy(assetDir, outDir, {
        overwrite: true,
        filter: (src) => !src.includes('node_modules'),
      });
      console.log(pc.gray(`  → ${assetDir}`));
    }
  }
}

/**
 * Generate index.html
 */
async function buildHtml(config: FerricConfig, outDir: string): Promise<void> {
  console.log(pc.gray(`Generating index.html...`));

  const html = await generateHtmlFromConfig(config);
  const outPath = path.join(outDir, 'index.html');

  await fs.writeFile(outPath, html, 'utf-8');
  console.log(pc.gray(`  → index.html`));
}

/**
 * Watch for file changes and rebuild
 */
async function watchForChanges(config: FerricConfig, outDir: string): Promise<void> {
  const srcDir = path.dirname(config.entry || './src/main.ts');
  const assetDirs = config.assets || ['public'];

  const watcher = chokidarWatch([
    `${srcDir}/**/*.{ts,tsx}`,
    ...assetDirs.map(d => `${d}/**/*`),
  ], {
    ignoreInitial: true,
    ignored: ['**/node_modules/**', '**/dist/**'],
  });

  watcher.on('change', async (filePath) => {
    console.log(pc.cyan(`\nFile changed: ${filePath}`));

    const startTime = Date.now();

    try {
      if (filePath.endsWith('.ts') || filePath.endsWith('.tsx')) {
        await buildTypeScript(config, outDir);
      } else {
        await buildAssets(config, outDir);
      }

      const elapsed = Date.now() - startTime;
      console.log(pc.green(`Rebuilt in ${elapsed}ms`));
    } catch (error) {
      console.error(pc.red('Rebuild failed:'), error);
    }
  });

  watcher.on('add', async (filePath) => {
    console.log(pc.cyan(`\nFile added: ${filePath}`));
    // Rebuild relevant section
    if (filePath.endsWith('.ts') || filePath.endsWith('.tsx')) {
      await buildTypeScript(config, outDir);
    } else {
      await buildAssets(config, outDir);
    }
  });

  // Keep process running
  await new Promise(() => {});
}

