/**
 * Configuration loader for Ferric projects
 */

import path from 'node:path';
import fs from 'fs-extra';

export interface FerricConfig {
  /** Project name */
  name?: string;

  /** Entry point for the application */
  entry?: string;

  /** Output directory */
  outDir?: string;

  /** Public path for assets */
  publicPath?: string;

  /** HTML template options */
  html?: {
    /** Title for the HTML page */
    title?: string;
    /** Path to custom template */
    template?: string;
    /** Favicon path */
    favicon?: string;
    /** Additional meta tags */
    meta?: Record<string, string>;
    /** Additional head content */
    headContent?: string;
    /** Additional body content (before scripts) */
    bodyContent?: string;
  };

  /** WASM options */
  wasm?: {
    /** Path to wasm-pack output */
    pkgDir?: string;
    /** WASM file name (without extension) */
    moduleName?: string;
  };

  /** SWC/Build options */
  build?: {
    /** Enable minification */
    minify?: boolean;
    /** Generate source maps */
    sourcemap?: boolean;
    /** Target browsers/environments */
    target?: string;
    /** Additional SWC options */
    swc?: Record<string, unknown>;
  };

  /** Dev server options */
  dev?: {
    /** Port number */
    port?: number;
    /** Host to bind */
    host?: string;
    /** Open browser on start */
    open?: boolean;
    /** Proxy configuration */
    proxy?: Record<string, string>;
  };

  /** Additional assets to copy */
  assets?: string[];
}

const DEFAULT_CONFIG: FerricConfig = {
  name: 'ferric-app',
  entry: './src/main.ts',
  outDir: 'dist',
  publicPath: '/',
  html: {
    title: 'Ferric App',
  },
  wasm: {
    pkgDir: './pkg',
    moduleName: 'ferric',
  },
  build: {
    minify: true,
    sourcemap: true,
    target: 'es2022',
  },
  dev: {
    port: 3000,
    host: 'localhost',
    open: false,
  },
  assets: ['public'],
};

/**
 * Load configuration from ferric.config.js or ferric.config.ts
 */
export async function loadConfig(configPath?: string): Promise<FerricConfig> {
  const cwd = process.cwd();

  // Try to find config file
  const configFiles = [
    configPath,
    'ferric.config.js',
    'ferric.config.mjs',
    'ferric.config.ts',
  ].filter(Boolean) as string[];

  let loadedConfig: Partial<FerricConfig> = {};

  for (const file of configFiles) {
    const fullPath = path.resolve(cwd, file);
    if (await fs.pathExists(fullPath)) {
      try {
        const module = await import(fullPath);
        loadedConfig = module.default || module;
        break;
      } catch (error) {
        // Try reading as JSON
        if (file.endsWith('.json')) {
          loadedConfig = await fs.readJson(fullPath);
          break;
        }
        throw error;
      }
    }
  }

  // Merge with defaults
  return mergeConfig(DEFAULT_CONFIG, loadedConfig);
}

/**
 * Deep merge configuration objects
 */
function mergeConfig(base: FerricConfig, override: Partial<FerricConfig>): FerricConfig {
  const result = { ...base };

  for (const key of Object.keys(override) as (keyof FerricConfig)[]) {
    const value = override[key];
    if (value !== undefined) {
      if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        (result as any)[key] = {
          ...(base[key] as object || {}),
          ...value,
        };
      } else {
        (result as any)[key] = value;
      }
    }
  }

  return result;
}

/**
 * Define configuration with type hints
 */
export function defineConfig(config: FerricConfig): FerricConfig {
  return config;
}

