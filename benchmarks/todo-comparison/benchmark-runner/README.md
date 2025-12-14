# Benchmark Runner

WASM-based benchmark runner for comparing Ferric vs React TodoMVC implementations.

## Building

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build the WASM module
wasm-pack build --target web --out-dir ../../web/pkg
```

## Usage

The benchmark runner provides:

### `BenchmarkRunner`

Main benchmark orchestration class:

```javascript
import { BenchmarkRunner } from './pkg/benchmark_runner.js';

const runner = new BenchmarkRunner();
runner.runTest("Ferric", "Add 100 Todos", 100, 12.5);
runner.runTest("React", "Add 100 Todos", 100, 18.3);

const suite = runner.generateSuite("TodoMVC Benchmarks");
console.log(suite);
```

### `TodoBenchmark`

Pre-built TodoMVC operation benchmarks:

```javascript
import { TodoBenchmark } from './pkg/benchmark_runner.js';

const addTime = TodoBenchmark.benchmarkAdd(100);
const toggleTime = TodoBenchmark.benchmarkToggle(100);
const deleteTime = TodoBenchmark.benchmarkDelete(100);
const filterTime = TodoBenchmark.benchmarkFilter(100);
```

## Integration with gh-pages

The benchmark results are displayed on the website at `/benchmarks.html`:

1. Run benchmarks via `/run-benchmarks.html`
2. Results are saved to localStorage
3. View results at `/benchmarks.html`

## Features

- ✅ Real-time performance measurement
- ✅ Memory usage estimation (Chrome only)
- ✅ Operations per second calculation
- ✅ Statistical summary generation
- ✅ JSON export for CI/CD integration
- ✅ Beautiful visualizations

## API Reference

### `BenchmarkResult`

```typescript
interface BenchmarkResult {
    framework: string;
    test: string;
    operations: number;
    time_ms: number;
    ops_per_second: number;
    memory_mb: number;
}
```

### `BenchmarkSuite`

```typescript
interface BenchmarkSuite {
    name: string;
    timestamp: string;
    results: BenchmarkResult[];
    summary: BenchmarkSummary;
}
```

### `BenchmarkSummary`

```typescript
interface BenchmarkSummary {
    ferric_avg_time: number;
    react_avg_time: number;
    ferric_faster_by: number;
    total_tests: number;
}
```

## License

MIT

