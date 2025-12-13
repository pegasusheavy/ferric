/**
 * HTML generator for Ferric projects
 */

import fs from 'fs-extra';
import path from 'node:path';
import type { FerricConfig } from '../config/loader.js';

export interface HtmlOptions {
  /** Page title */
  title?: string;
  /** Path to favicon */
  favicon?: string;
  /** Meta tags */
  meta?: Record<string, string>;
  /** CSS files to include */
  styles?: string[];
  /** JS files to include */
  scripts?: string[];
  /** WASM module path */
  wasmPath?: string;
  /** WASM JS bindings path */
  wasmBindingsPath?: string;
  /** Public path prefix */
  publicPath?: string;
  /** Additional content for <head> */
  headContent?: string;
  /** Additional content for <body> (before scripts) */
  bodyContent?: string;
  /** Root element selector */
  rootSelector?: string;
  /** Enable module preload hints */
  preload?: boolean;
  /** Custom template path */
  template?: string;
}

const DEFAULT_OPTIONS: HtmlOptions = {
  title: 'Ferric App',
  meta: {
    charset: 'UTF-8',
    viewport: 'width=device-width, initial-scale=1.0',
  },
  styles: [],
  scripts: [],
  publicPath: '/',
  rootSelector: 'app-root',
  preload: true,
};

/**
 * Generate index.html for a Ferric project
 */
export async function generateHtml(options: HtmlOptions = {}): Promise<string> {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  // Check for custom template
  if (opts.template && await fs.pathExists(opts.template)) {
    return processTemplate(opts.template, opts);
  }

  // Generate default HTML
  return generateDefaultHtml(opts);
}

/**
 * Generate the default HTML template
 */
function generateDefaultHtml(opts: HtmlOptions): string {
  const publicPath = opts.publicPath?.endsWith('/')
    ? opts.publicPath
    : `${opts.publicPath}/`;

  // Build meta tags
  const metaTags = Object.entries(opts.meta || {})
    .map(([key, value]) => {
      if (key === 'charset') {
        return `<meta charset="${value}">`;
      }
      return `<meta name="${key}" content="${value}">`;
    })
    .join('\n    ');

  // Build preload hints
  const preloadHints = opts.preload ? buildPreloadHints(opts, publicPath) : '';

  // Build style tags
  const styleTags = (opts.styles || [])
    .map(href => `<link rel="stylesheet" href="${publicPath}${href}">`)
    .join('\n    ');

  // Build script tags
  const scriptTags = (opts.scripts || [])
    .map(src => `<script type="module" src="${publicPath}${src}"></script>`)
    .join('\n    ');

  // Build WASM initialization script
  const wasmInit = buildWasmInitScript(opts, publicPath);

  // Favicon
  const faviconTag = opts.favicon
    ? `<link rel="icon" href="${publicPath}${opts.favicon}">`
    : '';

  return `<!DOCTYPE html>
<html lang="en">
  <head>
    ${metaTags}
    <title>${opts.title}</title>
    ${faviconTag}
    ${preloadHints}
    ${styleTags}
    ${opts.headContent || ''}
    <style>
      /* Ferric default styles */
      *, *::before, *::after {
        box-sizing: border-box;
      }

      html, body {
        margin: 0;
        padding: 0;
        min-height: 100vh;
      }

      ${opts.rootSelector} {
        display: block;
        min-height: 100vh;
      }

      /* Loading indicator */
      .ferric-loading {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
        font-family: system-ui, -apple-system, sans-serif;
        color: #666;
      }

      .ferric-loading::after {
        content: '';
        width: 32px;
        height: 32px;
        border: 3px solid #eee;
        border-top-color: #333;
        border-radius: 50%;
        animation: ferric-spin 0.8s linear infinite;
        margin-left: 12px;
      }

      @keyframes ferric-spin {
        to { transform: rotate(360deg); }
      }

      /* Hide loading after WASM init */
      .ferric-ready .ferric-loading {
        display: none;
      }
    </style>
  </head>
  <body>
    <${opts.rootSelector} class="ferric-loading">Loading...</${opts.rootSelector}>
    ${opts.bodyContent || ''}
    ${scriptTags}
    ${wasmInit}
  </body>
</html>`;
}

/**
 * Build preload hints for better performance
 */
function buildPreloadHints(opts: HtmlOptions, publicPath: string): string {
  const hints: string[] = [];

  // Preload WASM module
  if (opts.wasmPath) {
    hints.push(`<link rel="preload" href="${publicPath}${opts.wasmPath}" as="fetch" crossorigin>`);
  }

  // Preload JS bindings
  if (opts.wasmBindingsPath) {
    hints.push(`<link rel="modulepreload" href="${publicPath}${opts.wasmBindingsPath}">`);
  }

  // Preload scripts
  for (const script of opts.scripts || []) {
    hints.push(`<link rel="modulepreload" href="${publicPath}${script}">`);
  }

  return hints.join('\n    ');
}

/**
 * Build WASM initialization script
 */
function buildWasmInitScript(opts: HtmlOptions, publicPath: string): string {
  if (!opts.wasmBindingsPath) {
    return '';
  }

  const modulePath = `${publicPath}${opts.wasmBindingsPath}`;
  const wasmPath = opts.wasmPath ? `"${publicPath}${opts.wasmPath}"` : 'undefined';

  return `
    <script type="module">
      import init, * as ferric from '${modulePath}';

      // Initialize WASM module
      async function bootstrap() {
        try {
          await init(${wasmPath});

          // Mark as ready
          document.body.classList.add('ferric-ready');

          // Bootstrap the application
          if (typeof ferric.bootstrap === 'function') {
            ferric.bootstrap('${opts.rootSelector}');
          }

          // Expose to window for debugging
          if (import.meta.env?.DEV) {
            window.__FERRIC__ = ferric;
          }
        } catch (error) {
          console.error('Failed to initialize Ferric:', error);
          document.querySelector('${opts.rootSelector}').innerHTML =
            '<div style="color: red; padding: 20px;">Failed to load application. Check console for details.</div>';
        }
      }

      bootstrap();
    </script>`;
}

/**
 * Process a custom HTML template
 */
async function processTemplate(templatePath: string, opts: HtmlOptions): Promise<string> {
  let template = await fs.readFile(templatePath, 'utf-8');

  const publicPath = opts.publicPath?.endsWith('/')
    ? opts.publicPath
    : `${opts.publicPath}/`;

  // Replace template variables
  const replacements: Record<string, string> = {
    '{{title}}': opts.title || 'Ferric App',
    '{{publicPath}}': publicPath,
    '{{rootSelector}}': opts.rootSelector || 'app-root',
    '{{headContent}}': opts.headContent || '',
    '{{bodyContent}}': opts.bodyContent || '',
    '{{styles}}': (opts.styles || [])
      .map(href => `<link rel="stylesheet" href="${publicPath}${href}">`)
      .join('\n'),
    '{{scripts}}': (opts.scripts || [])
      .map(src => `<script type="module" src="${publicPath}${src}"></script>`)
      .join('\n'),
    '{{wasmInit}}': buildWasmInitScript(opts, publicPath),
  };

  for (const [key, value] of Object.entries(replacements)) {
    template = template.replaceAll(key, value);
  }

  return template;
}

/**
 * Generate HTML from Ferric config
 */
export async function generateHtmlFromConfig(config: FerricConfig): Promise<string> {
  const wasmPkgDir = config.wasm?.pkgDir || './pkg';
  const moduleName = config.wasm?.moduleName || 'ferric';

  return generateHtml({
    title: config.html?.title || config.name,
    favicon: config.html?.favicon,
    meta: config.html?.meta,
    headContent: config.html?.headContent,
    bodyContent: config.html?.bodyContent,
    template: config.html?.template,
    publicPath: config.publicPath,
    wasmPath: `${moduleName}_bg.wasm`,
    wasmBindingsPath: `${moduleName}.js`,
    preload: true,
  });
}

/**
 * Write generated HTML to file
 */
export async function writeHtml(
  outPath: string,
  options: HtmlOptions = {}
): Promise<void> {
  const html = await generateHtml(options);
  await fs.ensureDir(path.dirname(outPath));
  await fs.writeFile(outPath, html, 'utf-8');
}

