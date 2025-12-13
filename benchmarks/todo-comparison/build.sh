#!/bin/bash
set -e

echo "🔨 Building both Todo apps for benchmarking..."
echo ""

cd "$(dirname "$0")"

# Build React app
echo "📦 Building React Todo..."
cd react-todo
npm install
npm run build
cd ..
echo "✅ React build complete"
echo ""

# Build Ferric app
echo "🦀 Building Ferric Todo..."
cd ferric-todo
chmod +x build.sh
./build.sh
cd ..
echo "✅ Ferric build complete"
echo ""

echo "🎉 Both builds complete!"
echo ""
echo "To run the benchmark:"
echo "  1. Start React: cd react-todo && npm run preview"
echo "  2. Start Ferric: cd ferric-todo/dist && python3 -m http.server 3001"
echo "  3. Run benchmark: npm run benchmark"


set -e

echo "🔨 Building both Todo apps for benchmarking..."
echo ""

cd "$(dirname "$0")"

# Build React app
echo "📦 Building React Todo..."
cd react-todo
npm install
npm run build
cd ..
echo "✅ React build complete"
echo ""

# Build Ferric app
echo "🦀 Building Ferric Todo..."
cd ferric-todo
chmod +x build.sh
./build.sh
cd ..
echo "✅ Ferric build complete"
echo ""

echo "🎉 Both builds complete!"
echo ""
echo "To run the benchmark:"
echo "  1. Start React: cd react-todo && npm run preview"
echo "  2. Start Ferric: cd ferric-todo/dist && python3 -m http.server 3001"
echo "  3. Run benchmark: npm run benchmark"

