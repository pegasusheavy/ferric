/**
 * Development server command
 */

import path from 'node:path';
import fs from 'fs-extra';
import pc from 'picocolors';
import { loadConfig, type FerricConfig } from '../config/loader.js';
import { build } from './build.js';

export interface DevOptions {
  port?: string;
  open?: boolean;
  config?: string;
}

/**
 * Start development server
 */
export async function dev(options: DevOptions): Promise<void> {
  console.log(pc.cyan('\n🚀 Ferric Dev Server\n'));

  // Load configuration
  const config = await loadConfig(options.config);

  // Override with CLI options
  const port = parseInt(options.port || String(config.dev?.port || 3000));
  const shouldOpen = options.open ?? config.dev?.open ?? false;

  // Initial build
  console.log(pc.gray('Building project...\n'));
  await build({
    outdir: config.outDir,
    watch: false,
    minify: false,
    sourcemap: true,
    config: options.config,
  });

  // Start server using dynamic import (esbuild serve or similar)
  await startServer(config, port, shouldOpen);
}

/**
 * Start a simple development server
 */
async function startServer(
  config: FerricConfig,
  port: number,
  open: boolean
): Promise<void> {
  const outDir = config.outDir || 'dist';

  // Use Node.js built-in HTTP server
  const http = await import('node:http');
  const url = await import('node:url');

  const mimeTypes: Record<string, string> = {
    '.html': 'text/html',
    '.js': 'application/javascript',
    '.mjs': 'application/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.wasm': 'application/wasm',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon',
    '.woff': 'font/woff',
    '.woff2': 'font/woff2',
    '.ttf': 'font/ttf',
    '.eot': 'application/vnd.ms-fontobject',
  };

  const server = http.createServer(async (req, res) => {
    try {
      const parsedUrl = url.parse(req.url || '/', true);
      let pathname = parsedUrl.pathname || '/';

      // Default to index.html for root
      if (pathname === '/') {
        pathname = '/index.html';
      }

      // Resolve file path
      let filePath = path.join(outDir, pathname);

      // Check if file exists
      if (!await fs.pathExists(filePath)) {
        // Try with .html extension for SPA routing
        if (!path.extname(pathname)) {
          const htmlPath = path.join(outDir, `${pathname}.html`);
          if (await fs.pathExists(htmlPath)) {
            filePath = htmlPath;
          } else {
            // Fall back to index.html for SPA
            filePath = path.join(outDir, 'index.html');
          }
        } else {
          res.writeHead(404);
          res.end('Not Found');
          return;
        }
      }

      // Get file extension and MIME type
      const ext = path.extname(filePath).toLowerCase();
      const contentType = mimeTypes[ext] || 'application/octet-stream';

      // Read and serve file
      const content = await fs.readFile(filePath);

      // Add CORS and caching headers for development
      res.setHeader('Access-Control-Allow-Origin', '*');
      res.setHeader('Cache-Control', 'no-cache, no-store, must-revalidate');

      // Special headers for WASM
      if (ext === '.wasm') {
        res.setHeader('Content-Type', 'application/wasm');
      } else {
        res.setHeader('Content-Type', contentType);
      }

      res.writeHead(200);
      res.end(content);

      // Log request
      console.log(pc.gray(`${req.method} ${pathname}`));
    } catch (error) {
      console.error(pc.red('Server error:'), error);
      res.writeHead(500);
      res.end('Internal Server Error');
    }
  });

  server.listen(port, () => {
    const localUrl = `http://localhost:${port}`;

    console.log(pc.green('\n✓ Dev server running!\n'));
    console.log(`  ${pc.cyan('Local:')}   ${localUrl}`);
    console.log(`  ${pc.gray('Press Ctrl+C to stop')}\n`);

    // Open browser if requested
    if (open) {
      openBrowser(localUrl);
    }
  });

  // Watch for changes and rebuild
  console.log(pc.gray('Watching for changes...\n'));
  await build({
    outdir: config.outDir,
    watch: true,
    minify: false,
    sourcemap: true,
    config: undefined,
  });
}

/**
 * Open URL in default browser
 */
async function openBrowser(url: string): Promise<void> {
  const { exec } = await import('node:child_process');

  const command = process.platform === 'darwin'
    ? 'open'
    : process.platform === 'win32'
    ? 'start'
    : 'xdg-open';

  exec(`${command} ${url}`);
}

