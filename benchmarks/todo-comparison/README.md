# Todo App Benchmark: React vs Ferric

A comprehensive performance comparison between React and Ferric implementations of the same Todo application.

## Test Categories

### 1. Bundle Size
- Minified JavaScript/WASM size
- Gzipped size
- Initial CSS size

### 2. Runtime Performance
- Initial render time (1000 todos)
- Add todo (single item)
- Bulk add (100 todos)
- Toggle completion (single)
- Bulk toggle (100 todos)
- Delete todo
- Filter todos
- Clear completed

### 3. Memory Usage
- Initial memory footprint
- Memory with 1000 todos
- Memory after operations
- Garbage collection behavior

### 4. Interaction Latency
- Input responsiveness
- Click-to-update time
- Scroll performance (virtual list)

## Running the Benchmark

```bash
# Build both apps
./build.sh

# Run the benchmark suite
./run-benchmark.sh

# Generate report
./generate-report.sh
```

## Results

Results are stored in `results/` directory with timestamps.

## Requirements

- Node.js 18+
- Rust 1.70+
- wasm-pack
- Chromium (for Puppeteer benchmarks)



A comprehensive performance comparison between React and Ferric implementations of the same Todo application.

## Test Categories

### 1. Bundle Size
- Minified JavaScript/WASM size
- Gzipped size
- Initial CSS size

### 2. Runtime Performance
- Initial render time (1000 todos)
- Add todo (single item)
- Bulk add (100 todos)
- Toggle completion (single)
- Bulk toggle (100 todos)
- Delete todo
- Filter todos
- Clear completed

### 3. Memory Usage
- Initial memory footprint
- Memory with 1000 todos
- Memory after operations
- Garbage collection behavior

### 4. Interaction Latency
- Input responsiveness
- Click-to-update time
- Scroll performance (virtual list)

## Running the Benchmark

```bash
# Build both apps
./build.sh

# Run the benchmark suite
./run-benchmark.sh

# Generate report
./generate-report.sh
```

## Results

Results are stored in `results/` directory with timestamps.

## Requirements

- Node.js 18+
- Rust 1.70+
- wasm-pack
- Chromium (for Puppeteer benchmarks)

