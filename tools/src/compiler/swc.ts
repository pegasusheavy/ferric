/**
 * SWC compiler integration for TypeScript/JavaScript
 */

import { transform, type Options as SwcOptions, type ModuleConfig } from '@swc/core';
import fs from 'fs-extra';
import path from 'node:path';
import { glob } from 'glob';
import type { FerricConfig } from '../config/loader.js';

export interface CompileOptions {
  /** Source files or glob patterns */
  input: string | string[];
  /** Output directory */
  outDir: string;
  /** Enable minification */
  minify?: boolean;
  /** Generate source maps */
  sourcemap?: boolean;
  /** Target environment */
  target?: string;
  /** Additional SWC options */
  swcOptions?: Partial<SwcOptions>;
}

/**
 * Compile TypeScript/JavaScript files using SWC
 */
export async function compileTypeScript(options: CompileOptions): Promise<void> {
  const {
    input,
    outDir,
    minify = false,
    sourcemap = true,
    target = 'es2022',
    swcOptions = {},
  } = options;

  // Resolve input files
  const patterns = Array.isArray(input) ? input : [input];
  const files: string[] = [];

  for (const pattern of patterns) {
    const matches = await glob(pattern, {
      ignore: ['**/node_modules/**', '**/dist/**'],
    });
    files.push(...matches);
  }

  // Ensure output directory exists
  await fs.ensureDir(outDir);

  // Compile each file
  const swcConfig: SwcOptions = {
    jsc: {
      parser: {
        syntax: 'typescript',
        tsx: true,
        decorators: true,
        dynamicImport: true,
      },
      target: target as any,
      loose: false,
      externalHelpers: false,
      keepClassNames: true,
      transform: {
        legacyDecorator: true,
        decoratorMetadata: true,
      },
      ...swcOptions.jsc,
    },
    module: {
      type: 'nodenext',
      ...swcOptions.module,
    } as ModuleConfig,
    minify,
    sourceMaps: sourcemap,
    ...swcOptions,
  };

  const results = await Promise.all(
    files.map(async (file) => {
      const source = await fs.readFile(file, 'utf-8');
      const result = await transform(source, {
        ...swcConfig,
        filename: file,
      });

      // Determine output path
      const relativePath = path.relative(process.cwd(), file);
      const outPath = path.join(
        outDir,
        relativePath.replace(/\.tsx?$/, '.js')
      );

      // Write output
      await fs.ensureDir(path.dirname(outPath));
      await fs.writeFile(outPath, result.code);

      if (result.map && sourcemap) {
        await fs.writeFile(`${outPath}.map`, result.map);
      }

      return { input: file, output: outPath };
    })
  );

  return;
}

/**
 * Bundle files using SWC
 */
export async function bundleFiles(
  entry: string,
  outFile: string,
  config: FerricConfig
): Promise<void> {
  const minify = config.build?.minify ?? true;

  // Read entry file
  const source = await fs.readFile(entry, 'utf-8');

  // Transform with SWC
  const result = await transform(source, {
    jsc: {
      parser: {
        syntax: 'typescript',
        tsx: true,
        decorators: true,
      },
      target: (config.build?.target as any) || 'es2022',
      minify: minify ? {
        compress: true,
        mangle: true,
      } : undefined,
    },
    module: {
      type: 'nodenext',
    } as ModuleConfig,
    minify,
    sourceMaps: config.build?.sourcemap ?? true,
    filename: entry,
  });

  await fs.ensureDir(path.dirname(outFile));
  await fs.writeFile(outFile, result.code);

  if (result.map) {
    await fs.writeFile(`${outFile}.map`, result.map);
  }
}

/**
 * Get default SWC configuration for Ferric projects
 */
export function getDefaultSwcConfig(): SwcOptions {
  return {
    jsc: {
      parser: {
        syntax: 'typescript',
        tsx: true,
        decorators: true,
        dynamicImport: true,
      },
      target: 'es2022',
      loose: false,
      externalHelpers: false,
      keepClassNames: true,
    },
    module: {
      type: 'nodenext',
    } as ModuleConfig,
    sourceMaps: true,
  };
}

