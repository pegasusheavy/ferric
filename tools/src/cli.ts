#!/usr/bin/env node
/**
 * Ferric CLI - Build tools for Ferric framework projects
 */

import { Command } from 'commander';
import { build } from './commands/build.js';
import { dev } from './commands/dev.js';
import { init } from './commands/init.js';
import pc from 'picocolors';

const program = new Command();

program
  .name('ferric')
  .description('Build tools for Ferric WASM framework')
  .version('0.1.0');

program
  .command('build')
  .description('Build the Ferric project for production')
  .option('-o, --outdir <dir>', 'Output directory', 'dist')
  .option('-w, --watch', 'Watch for changes and rebuild')
  .option('--no-minify', 'Disable minification')
  .option('--no-sourcemap', 'Disable source maps')
  .option('-c, --config <file>', 'Path to ferric.config.js')
  .action(async (options) => {
    try {
      await build(options);
    } catch (error) {
      console.error(pc.red('Build failed:'), error);
      process.exit(1);
    }
  });

program
  .command('dev')
  .description('Start development server with hot reload')
  .option('-p, --port <port>', 'Port to listen on', '3000')
  .option('-o, --open', 'Open browser automatically')
  .option('-c, --config <file>', 'Path to ferric.config.js')
  .action(async (options) => {
    try {
      await dev(options);
    } catch (error) {
      console.error(pc.red('Dev server failed:'), error);
      process.exit(1);
    }
  });

program
  .command('init')
  .description('Initialize a new Ferric project')
  .argument('[name]', 'Project name')
  .option('-t, --template <template>', 'Template to use', 'default')
  .action(async (name, options) => {
    try {
      await init(name, options);
    } catch (error) {
      console.error(pc.red('Init failed:'), error);
      process.exit(1);
    }
  });

program.parse();

