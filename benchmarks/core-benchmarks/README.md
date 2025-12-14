# Ferric Core Benchmarks

Comprehensive benchmarks for the Ferric framework core components.

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench --package core-benchmarks
```

### Run Specific Benchmark Suite

```bash
# Reactivity benchmarks
cargo bench --bench reactivity

# Component benchmarks
cargo bench --bench component

# DOM benchmarks
cargo bench --bench dom

# Router benchmarks
cargo bench --bench router

# Forms benchmarks
cargo bench --bench forms

# Pipes benchmarks
cargo bench --bench pipes
```

### Run Specific Test

```bash
cargo bench --bench reactivity -- signal_creation
```

## Benchmark Suites

### 1. Reactivity Benchmarks (`reactivity`)

Tests the performance of the reactive system:

- **Signal Operations**: Creation, get, set, update
- **Computed Values**: Creation, get, chaining
- **Effects**: Creation, propagation
- **Batch Updates**: Multiple signal updates
- **Scale Tests**: Many signals/computed (10, 100, 1000)
- **Diamond Dependencies**: Complex dependency graphs

### 2. Component Benchmarks (`component`)

Tests component lifecycle and operations:

- **Metadata Creation**: Component metadata initialization
- **Context Creation**: Component context setup
- **Input Bindings**: Input property updates (1, 10, 50 inputs)
- **Output Emissions**: Event emitter performance (1, 10, 50 outputs)
- **Lifecycle Hooks**: OnInit, OnDestroy execution
- **Nested Components**: Component hierarchy (1, 5, 10 levels)

### 3. DOM Benchmarks (`dom`)

Tests DOM manipulation performance:

- **Element Creation**: Virtual DOM element creation
- **Attribute Setting**: Multiple attribute updates (1, 10, 50)
- **Text Updates**: Text content modifications
- **Class Operations**: Class list manipulation (1, 10, 50 classes)
- **Style Updates**: Style property updates (1, 10, 50 styles)
- **Event Listeners**: Event registration
- **Virtual DOM Diffing**: Change detection simulation

### 4. Router Benchmarks (`router`)

Tests routing performance:

- **Router Creation**: Router initialization
- **Route Matching**: URL pattern matching
- **Parameter Extraction**: Route params parsing
- **Query Parameters**: Query string parsing
- **Scale Tests**: Many routes (10, 50, 100)
- **Nested Routes**: Route hierarchies

### 5. Forms Benchmarks (`forms`)

Tests forms system performance:

- **Control Creation**: FormControl initialization
- **Value Updates**: Control value changes
- **Validation**: Validator execution
- **FormGroup Operations**: Group creation (5, 10, 20 controls)
- **FormGroup Values**: Getting group values
- **FormGroup Validation**: Validating groups
- **FormArray Operations**: Array modifications
- **Nested Groups**: Deep form hierarchies (2, 5, 10 levels)
- **Built-in Validators**: Required, minLength, email, etc.

### 6. Pipes Benchmarks (`pipes`)

Tests pipe transformation performance:

- **Registry Operations**: Pipe registration and lookup
- **Built-in Pipes**: UpperCase, LowerCase, TitleCase, JSON, Date, Number
- **Pipe Chaining**: Sequential pipe applications
- **Scale Tests**: Many pipe invocations (10, 50, 100)
- **Text Length**: Long text processing (100, 1000, 10000 chars)

## Interpreting Results

Criterion generates detailed HTML reports in `target/criterion/`:

```bash
# Open the index page
open target/criterion/report/index.html
```

Each benchmark shows:
- **Time**: Mean execution time with confidence intervals
- **Throughput**: Operations per second
- **Comparison**: Performance vs previous runs
- **Plots**: Time series and probability distributions

## Performance Guidelines

### Reactivity System
- Signal creation: < 100ns
- Signal get/set: < 50ns
- Computed evaluation: < 200ns
- Effect propagation: < 500ns

### Components
- Metadata creation: < 1µs
- Context creation: < 2µs
- Lifecycle hooks: < 100ns per hook

### Forms
- Control creation: < 500ns
- Validation: < 1µs per validator
- Group operations: Linear with control count

### Pipes
- Simple transforms: < 500ns
- Complex transforms: < 5µs

## Adding New Benchmarks

1. Create a new bench file in `benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_your_feature(c: &mut Criterion) {
    c.bench_function("your_feature", |b| {
        b.iter(|| {
            // Your benchmark code
            black_box(some_operation());
        });
    });
}

criterion_group!(benches, bench_your_feature);
criterion_main!(benches);
```

2. Add to `Cargo.toml`:

```toml
[[bench]]
name = "your_feature"
harness = false
```

3. Run your benchmark:

```bash
cargo bench --bench your_feature
```

## Continuous Benchmarking

For CI/CD integration:

```bash
# Generate baseline
cargo bench --bench reactivity -- --save-baseline main

# Compare against baseline
cargo bench --bench reactivity -- --baseline main
```

## Tips

1. **Reduce Noise**: Close other applications, disable CPU frequency scaling
2. **Multiple Runs**: Criterion runs each test many times for statistical accuracy
3. **Warm-up**: Benchmarks include warm-up iterations
4. **Black Box**: Use `black_box()` to prevent compiler optimizations
5. **Realistic Data**: Use representative data sizes and patterns

## License

MIT

