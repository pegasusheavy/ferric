//! Benchmarks for Ferric's reactive system.
//!
//! Run with: cargo bench -p ferric

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ferric_core::reactive::{batch, computed, signal};

/// Benchmark signal creation
fn bench_signal_creation(c: &mut Criterion) {
    c.bench_function("signal_create", |b| {
        b.iter(|| {
            let s = signal(black_box(42));
            black_box(s)
        })
    });
}

/// Benchmark signal get operations
fn bench_signal_get(c: &mut Criterion) {
    let s = signal(42);

    c.bench_function("signal_get", |b| {
        b.iter(|| black_box(s.get()))
    });
}

/// Benchmark signal set operations
fn bench_signal_set(c: &mut Criterion) {
    let s = signal(0);

    c.bench_function("signal_set", |b| {
        b.iter(|| {
            s.set(black_box(42));
        })
    });
}

/// Benchmark signal update operations
fn bench_signal_update(c: &mut Criterion) {
    let s = signal(0);

    c.bench_function("signal_update", |b| {
        b.iter(|| {
            s.update(|n| n + 1);
        })
    });
}

/// Benchmark computed value creation
fn bench_computed_creation(c: &mut Criterion) {
    c.bench_function("computed_create", |b| {
        b.iter(|| {
            let c = computed(|| black_box(42));
            black_box(c)
        })
    });
}

/// Benchmark computed value get (first access - computation)
fn bench_computed_get_first(c: &mut Criterion) {
    c.bench_function("computed_get_first", |b| {
        b.iter(|| {
            let c = computed(|| 42);
            black_box(c.get())
        })
    });
}

/// Benchmark computed value get (cached)
fn bench_computed_get_cached(crit: &mut Criterion) {
    let comp = computed(|| 42);
    // Prime the cache
    let _ = comp.get();

    crit.bench_function("computed_get_cached", |b| {
        b.iter(|| black_box(comp.get()))
    });
}

/// Benchmark computed with signal dependency
fn bench_computed_with_signal(c: &mut Criterion) {
    let count = signal(0);
    let doubled = computed({
        let count = count.clone();
        move || count.get() * 2
    });

    c.bench_function("computed_with_signal", |b| {
        b.iter(|| {
            count.set(black_box(42));
            black_box(doubled.get())
        })
    });
}

/// Benchmark dependency chain propagation
fn bench_dependency_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_chain");

    for chain_len in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*chain_len as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(chain_len),
            chain_len,
            |b, &len| {
                let source = signal(0);

                // Build chain of computed values
                let mut computeds = Vec::with_capacity(len);
                let first = computed({
                    let source = source.clone();
                    move || source.get() + 1
                });
                computeds.push(first);

                for i in 1..len {
                    let prev = computeds[i - 1].clone();
                    let c = computed(move || prev.get() + 1);
                    computeds.push(c);
                }

                let last = computeds.last().unwrap().clone();

                b.iter(|| {
                    source.set(black_box(42));
                    black_box(last.get())
                })
            },
        );
    }
    group.finish();
}

/// Benchmark wide dependency fan-out
fn bench_fan_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("fan_out");

    for fan_width in [1, 5, 10, 50].iter() {
        group.throughput(Throughput::Elements(*fan_width as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(fan_width),
            fan_width,
            |b, &width| {
                let source = signal(0);

                // Create many computed values depending on the same signal
                let computeds: Vec<_> = (0..width)
                    .map(|i| {
                        let source = source.clone();
                        computed(move || source.get() + i)
                    })
                    .collect();

                b.iter(|| {
                    source.set(black_box(42));
                    // Read all computed values
                    let sum: i32 = computeds.iter().map(|c| c.get()).sum();
                    black_box(sum)
                })
            },
        );
    }
    group.finish();
}

/// Benchmark batched updates
fn bench_batch_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_updates");

    for num_signals in [1, 5, 10, 50].iter() {
        group.throughput(Throughput::Elements(*num_signals as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_signals),
            num_signals,
            |b, &count| {
                let signals: Vec<_> = (0..count).map(|_| signal(0)).collect();
                let sum = computed({
                    let signals = signals.clone();
                    move || signals.iter().map(|s| s.get()).sum::<i32>()
                });

                b.iter(|| {
                    batch(|| {
                        for (i, s) in signals.iter().enumerate() {
                            s.set(black_box(i as i32));
                        }
                    });
                    black_box(sum.get())
                })
            },
        );
    }
    group.finish();
}

/// Benchmark unbatched updates for comparison
fn bench_unbatched_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("unbatched_updates");

    for num_signals in [1, 5, 10, 50].iter() {
        group.throughput(Throughput::Elements(*num_signals as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_signals),
            num_signals,
            |b, &count| {
                let signals: Vec<_> = (0..count).map(|_| signal(0)).collect();
                let sum = computed({
                    let signals = signals.clone();
                    move || signals.iter().map(|s| s.get()).sum::<i32>()
                });

                b.iter(|| {
                    for (i, s) in signals.iter().enumerate() {
                        s.set(black_box(i as i32));
                    }
                    black_box(sum.get())
                })
            },
        );
    }
    group.finish();
}

/// Benchmark many signals being read by one computed
fn bench_computed_many_deps(c: &mut Criterion) {
    let mut group = c.benchmark_group("computed_many_deps");

    for num_deps in [1, 5, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*num_deps as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_deps),
            num_deps,
            |b, &count| {
                let signals: Vec<_> = (0..count).map(|i| signal(i as i32)).collect();
                let sum = computed({
                    let signals = signals.clone();
                    move || signals.iter().map(|s| s.get()).sum::<i32>()
                });

                // Prime the cache
                let _ = sum.get();

                b.iter(|| black_box(sum.get()))
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_signal_creation,
    bench_signal_get,
    bench_signal_set,
    bench_signal_update,
    bench_computed_creation,
    bench_computed_get_first,
    bench_computed_get_cached,
    bench_computed_with_signal,
    bench_dependency_chain,
    bench_fan_out,
    bench_batch_updates,
    bench_unbatched_updates,
    bench_computed_many_deps,
);

criterion_main!(benches);

