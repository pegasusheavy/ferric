#!/bin/bash

# Build script for GitHub Pages deployment
# This builds the Ferric documentation site with all assets

set -e

echo "🦀 Building Ferric Documentation Site for GitHub Pages"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Change to project root
cd "$(dirname "$0")/.."

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}❌ wasm-pack not found. Installing...${NC}"
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js not found. Please install Node.js 18+${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites OK${NC}"
echo ""

# Install Node dependencies
echo -e "${YELLOW}📦 Installing Node dependencies...${NC}"
cd web
npm install
cd ..
echo -e "${GREEN}✅ Dependencies installed${NC}"
echo ""

# Build CSS with TailwindCSS 4
echo -e "${YELLOW}🎨 Building CSS with TailwindCSS 4...${NC}"
cd web
npx @tailwindcss/cli@next -i ./styles/rust-theme.css -o ./styles/dist/rust-theme.min.css --minify
cd ..
echo -e "${GREEN}✅ CSS built${NC}"
echo ""

# Build Wasm application
echo -e "${YELLOW}🦀 Building Wasm application...${NC}"
cd web
wasm-pack build --target web --out-dir pkg --release
cd ..
echo -e "${GREEN}✅ Wasm built${NC}"
echo ""

# Generate documentation HTML
echo -e "${YELLOW}📚 Generating documentation HTML...${NC}"
if [ -f scripts/generate-docs-html.sh ]; then
    bash scripts/generate-docs-html.sh
    echo -e "${GREEN}✅ Documentation generated${NC}"
else
    echo -e "${YELLOW}⚠️  No documentation generator found, skipping${NC}"
fi
echo ""

# Prepare deployment directory
echo -e "${YELLOW}📁 Preparing deployment directory...${NC}"
rm -rf deploy
mkdir -p deploy

# Copy all web assets
cp web/index.html deploy/
cp -r web/pkg deploy/
cp -r web/styles deploy/

# Copy optional assets
[ -f web/docs.html ] && cp web/docs.html deploy/
[ -f web/docs-rust-theme.html ] && cp web/docs-rust-theme.html deploy/
[ -f web/benchmarks.html ] && cp web/benchmarks.html deploy/
[ -f web/run-benchmarks.html ] && cp web/run-benchmarks.html deploy/
[ -d web/assets ] && cp -r web/assets deploy/
[ -d web/docs ] && cp -r web/docs deploy/

# Create .nojekyll to disable Jekyll
touch deploy/.nojekyll

# Create a basic README for gh-pages
cat > deploy/README.md << 'EOF'
# Ferric Framework Documentation

This branch contains the built documentation site for the Ferric Framework.

**[View Live Documentation →](https://pegasusheavy.github.io/ferric/)**

## About Ferric

Ferric is a modern web framework for Rust with Angular-inspired architecture, 
reactive programming, and WebAssembly performance.

## Local Development

To build this site locally:

```bash
# From the develop branch
git checkout develop

# Run the build script
bash scripts/build-for-gh-pages.sh

# Serve locally
cd deploy
python3 -m http.server 8080
```

## Contributing

Please make changes in the `develop` branch, not in `gh-pages`. 
The `gh-pages` branch is automatically built and deployed by GitHub Actions.

---

Built with 🦀 Rust and ❤️ by the Ferric community
EOF

echo -e "${GREEN}✅ Deployment directory ready${NC}"
echo ""

# Display summary
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✅ Build Complete!                    ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo "📦 Deployment ready in: ./deploy"
echo ""
echo "📊 Files included:"
ls -lh deploy/ | tail -n +2
echo ""
echo "📏 Total size:"
du -sh deploy/
echo ""
echo "🚀 To deploy manually:"
echo "   1. git checkout gh-pages"
echo "   2. rm -rf * .github"
echo "   3. cp -r deploy/* ."
echo "   4. git add -A"
echo "   5. git commit -m 'Deploy documentation'"
echo "   6. git push origin gh-pages"
echo ""
echo "💡 Or let GitHub Actions do it automatically!"

