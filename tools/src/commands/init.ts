/**
 * Project initialization command
 */

import path from 'node:path';
import fs from 'fs-extra';
import pc from 'picocolors';

export interface InitOptions {
  template?: string;
}

/**
 * Initialize a new Ferric project
 */
export async function init(name?: string, options: InitOptions = {}): Promise<void> {
  const projectName = name || 'ferric-app';
  const projectDir = path.resolve(process.cwd(), projectName);

  console.log(pc.cyan(`\n🚀 Creating Ferric project: ${projectName}\n`));

  // Check if directory exists
  if (await fs.pathExists(projectDir)) {
    const isEmpty = (await fs.readdir(projectDir)).length === 0;
    if (!isEmpty) {
      console.error(pc.red(`Directory ${projectName} already exists and is not empty`));
      process.exit(1);
    }
  }

  // Create project structure
  await fs.ensureDir(projectDir);

  console.log(pc.gray('Creating project structure...'));

  // Create directories
  const dirs = ['src', 'public', 'tests'];
  for (const dir of dirs) {
    await fs.ensureDir(path.join(projectDir, dir));
    console.log(pc.gray(`  → ${dir}/`));
  }

  // Create files
  await createProjectFiles(projectDir, projectName);

  console.log(pc.green('\n✓ Project created successfully!\n'));
  console.log('Next steps:');
  console.log(pc.cyan(`  cd ${projectName}`));
  console.log(pc.cyan('  pnpm install'));
  console.log(pc.cyan('  wasm-pack build --target web'));
  console.log(pc.cyan('  pnpm dev'));
  console.log('');
}

/**
 * Create project files
 */
async function createProjectFiles(projectDir: string, projectName: string): Promise<void> {
  // ferric.config.js
  const configContent = `// Ferric configuration
import { defineConfig } from '@ferric/tools';

export default defineConfig({
  name: '${projectName}',
  entry: './src/main.ts',
  outDir: 'dist',

  html: {
    title: '${projectName}',
    meta: {
      description: 'A Ferric application',
    },
  },

  wasm: {
    pkgDir: './pkg',
    moduleName: '${projectName.replace(/-/g, '_')}',
  },

  build: {
    minify: true,
    sourcemap: true,
    target: 'es2022',
  },

  dev: {
    port: 3000,
    open: true,
  },
});
`;
  await writeFile(projectDir, 'ferric.config.js', configContent);

  // package.json
  const packageJson = {
    name: projectName,
    version: '0.1.0',
    type: 'module',
    scripts: {
      dev: 'ferric dev',
      build: 'wasm-pack build --target web && ferric build',
      'build:wasm': 'wasm-pack build --target web',
      'build:js': 'ferric build',
      preview: 'ferric dev --port 4000',
    },
    dependencies: {
      '@ferric/tools': 'workspace:*',
    },
    devDependencies: {
      typescript: '^5.3.3',
    },
  };
  await writeFile(projectDir, 'package.json', JSON.stringify(packageJson, null, 2));

  // tsconfig.json
  const tsConfig = {
    compilerOptions: {
      target: 'ES2022',
      module: 'ESNext',
      moduleResolution: 'bundler',
      strict: true,
      esModuleInterop: true,
      skipLibCheck: true,
      forceConsistentCasingInFileNames: true,
      declaration: true,
      declarationMap: true,
      sourceMap: true,
      outDir: './dist',
      rootDir: './src',
    },
    include: ['src/**/*'],
    exclude: ['node_modules', 'dist', 'pkg'],
  };
  await writeFile(projectDir, 'tsconfig.json', JSON.stringify(tsConfig, null, 2));

  // Cargo.toml
  const cargoToml = `[package]
name = "${projectName.replace(/-/g, '_')}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
ferric = { path = "../ferric" }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["console", "Window", "Document", "Element"] }
console_error_panic_hook = "0.1"

[profile.release]
opt-level = "s"
lto = true
`;
  await writeFile(projectDir, 'Cargo.toml', cargoToml);

  // src/lib.rs
  const libRs = `//! ${projectName} - A Ferric application

use ferric::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    // Bootstrap the application
    ferric::bootstrap("app-root").expect("Failed to bootstrap");
}

#[component(
    selector = "app-root",
    template = r#"
        <div class="app">
            <h1>Welcome to {{ title }}!</h1>
            <p>Count: {{ count }}</p>
            <button (click)="increment()">Increment</button>
        </div>
    "#,
    styles = r#"
        .app {
            font-family: system-ui, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            text-align: center;
        }

        h1 {
            color: #333;
        }

        button {
            padding: 0.5rem 1rem;
            font-size: 1rem;
            cursor: pointer;
        }
    "#
)]
pub struct AppComponent {
    title: Signal<String>,
    count: Signal<i32>,
}

impl AppComponent {
    pub fn new() -> Self {
        Self {
            title: signal("${projectName}".to_string()),
            count: signal(0),
        }
    }

    pub fn increment(&self) {
        self.count.update(|n| n + 1);
    }
}
`;
  await writeFile(projectDir, 'src/lib.rs', libRs);

  // src/main.ts (optional TypeScript entry)
  const mainTs = `/**
 * TypeScript entry point for ${projectName}
 *
 * This file is optional - the WASM module handles bootstrapping.
 * Use this for additional JavaScript/TypeScript initialization.
 */

// The WASM module is loaded via the generated index.html
// Add any additional client-side JavaScript here

console.log('${projectName} loaded');
`;
  await writeFile(projectDir, 'src/main.ts', mainTs);

  // public/favicon.ico placeholder
  // (Just create an empty file - users can replace it)
  await writeFile(projectDir, 'public/.gitkeep', '');

  // .gitignore
  const gitignore = `/target
/pkg
/dist
/node_modules
Cargo.lock
*.log
.DS_Store
`;
  await writeFile(projectDir, '.gitignore', gitignore);

  // README.md
  const readme = `# ${projectName}

A web application built with the Ferric framework.

## Development

\`\`\`bash
# Install dependencies
pnpm install

# Build WASM module
wasm-pack build --target web

# Start dev server
pnpm dev
\`\`\`

## Production Build

\`\`\`bash
pnpm build
\`\`\`

The built files will be in the \`dist/\` directory.

## Project Structure

\`\`\`
${projectName}/
├── src/
│   ├── lib.rs          # Rust/WASM entry point
│   └── main.ts         # TypeScript entry point
├── public/             # Static assets
├── pkg/                # WASM build output (generated)
├── dist/               # Production build output
├── Cargo.toml          # Rust dependencies
├── ferric.config.js    # Build configuration
└── package.json
\`\`\`
`;
  await writeFile(projectDir, 'README.md', readme);
}

/**
 * Write a file with logging
 */
async function writeFile(dir: string, filename: string, content: string): Promise<void> {
  const filePath = path.join(dir, filename);
  await fs.ensureDir(path.dirname(filePath));
  await fs.writeFile(filePath, content, 'utf-8');
  console.log(pc.gray(`  → ${filename}`));
}

