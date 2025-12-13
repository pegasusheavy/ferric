#!/bin/bash
set -e

echo "Building Ferric Todo for benchmark..."

# Build with wasm-pack
wasm-pack build --target web --release --out-dir pkg

# Copy static files to dist
mkdir -p dist
cp index.html dist/
cp styles.css dist/
cp -r pkg dist/

echo "Build complete! Output in dist/"


set -e

echo "Building Ferric Todo for benchmark..."

# Build with wasm-pack
wasm-pack build --target web --release --out-dir pkg

# Copy static files to dist
mkdir -p dist
cp index.html dist/
cp styles.css dist/
cp -r pkg dist/

echo "Build complete! Output in dist/"

