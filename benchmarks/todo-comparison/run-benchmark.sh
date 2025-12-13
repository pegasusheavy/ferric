#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "🚀 Starting benchmark servers..."

# Start React preview server in background
echo "Starting React server on port 3000..."
cd react-todo
npm run preview &
REACT_PID=$!
cd ..

# Start Ferric server in background
echo "Starting Ferric server on port 3001..."
cd ferric-todo/dist
python3 -m http.server 3001 &
FERRIC_PID=$!
cd ../..

# Wait for servers to start
echo "Waiting for servers to start..."
sleep 3

# Install benchmark dependencies if needed
if [ ! -d "node_modules" ]; then
  echo "Installing benchmark dependencies..."
  npm install
fi

# Run the benchmark
echo ""
echo "🏃 Running benchmarks..."
npm run benchmark

# Cleanup
echo ""
echo "Cleaning up servers..."
kill $REACT_PID 2>/dev/null || true
kill $FERRIC_PID 2>/dev/null || true

echo "✅ Done! Check results/ for the benchmark report."


set -e

cd "$(dirname "$0")"

echo "🚀 Starting benchmark servers..."

# Start React preview server in background
echo "Starting React server on port 3000..."
cd react-todo
npm run preview &
REACT_PID=$!
cd ..

# Start Ferric server in background
echo "Starting Ferric server on port 3001..."
cd ferric-todo/dist
python3 -m http.server 3001 &
FERRIC_PID=$!
cd ../..

# Wait for servers to start
echo "Waiting for servers to start..."
sleep 3

# Install benchmark dependencies if needed
if [ ! -d "node_modules" ]; then
  echo "Installing benchmark dependencies..."
  npm install
fi

# Run the benchmark
echo ""
echo "🏃 Running benchmarks..."
npm run benchmark

# Cleanup
echo ""
echo "Cleaning up servers..."
kill $REACT_PID 2>/dev/null || true
kill $FERRIC_PID 2>/dev/null || true

echo "✅ Done! Check results/ for the benchmark report."

