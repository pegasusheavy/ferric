//! Benchmarks for pipes operations.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use ferric_core::pipes::{Pipe, PipeArgs, PipeRegistry};

fn bench_pipe_registry(c: &mut Criterion) {
    c.bench_function("pipe_registry_creation", |b| {
        b.iter(|| {
            black_box(PipeRegistry::new());
        });
    });
}

fn bench_string_transformation(c: &mut Criterion) {
    c.bench_function("string_uppercase", |b| {
        b.iter(|| {
            black_box("hello world".to_uppercase());
        });
    });

    c.bench_function("string_lowercase", |b| {
        b.iter(|| {
            black_box("HELLO WORLD".to_lowercase());
        });
    });
}

fn bench_pipe_simulation(c: &mut Criterion) {
    c.bench_function("pipe_transform_simulation", |b| {
        b.iter(|| {
            // Simulate pipe transformation
            let input = black_box("hello world");
            let result = input.to_uppercase();
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    bench_pipe_registry,
    bench_string_transformation,
    bench_pipe_simulation,
);

criterion_main!(benches);

