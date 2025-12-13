/**
 * Ferric Build Tools
 *
 * Programmatic API for building Ferric projects
 */

export { build, type BuildOptions } from './commands/build.js';
export { dev, type DevOptions } from './commands/dev.js';
export { init, type InitOptions } from './commands/init.js';
export { generateHtml, type HtmlOptions } from './html/generator.js';
export { loadConfig, type FerricConfig } from './config/loader.js';
export { compileTypeScript } from './compiler/swc.js';

