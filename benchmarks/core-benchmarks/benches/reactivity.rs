//! Benchmarks for the reactivity system.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ferric_core::reactive::{signal, computed, effect, batch};

fn bench_signal_creation(c: &mut Criterion) {
    c.bench_function("signal_creation", |b| {
        b.iter(|| {
            black_box(signal(42));
        });
    });
}

fn bench_signal_get(c: &mut Criterion) {
    let sig = signal(42);

    c.bench_function("signal_get", |b| {
        b.iter(|| {
            black_box(sig.get());
        });
    });
}

fn bench_signal_set(c: &mut Criterion) {
    let sig = signal(0);

    c.bench_function("signal_set", |b| {
        let mut i = 0;
        b.iter(|| {
            sig.set(i);
            i += 1;
        });
    });
}

fn bench_signal_update(c: &mut Criterion) {
    let sig = signal(0);

    c.bench_function("signal_update", |b| {
        b.iter(|| {
            sig.update(|val| *val += 1);
        });
    });
}

fn bench_computed_creation(c: &mut Criterion) {
    let sig = signal(42);

    c.bench_function("computed_creation", |b| {
        b.iter(|| {
            black_box(computed(|| sig.get() * 2));
        });
    });
}

fn bench_computed_get(c: &mut Criterion) {
    let sig = signal(42);
    let comp = computed(|| sig.get() * 2);

    c.bench_function("computed_get", |b| {
        b.iter(|| {
            black_box(comp.get());
        });
    });
}

fn bench_computed_chain(c: &mut Criterion) {
    let sig = signal(1);
    let comp1 = computed(|| sig.get() * 2);
    let comp2 = computed(|| comp1.get() + 10);
    let comp3 = computed(|| comp2.get() * 3);

    c.bench_function("computed_chain", |b| {
        b.iter(|| {
            black_box(comp3.get());
        });
    });
}

fn bench_effect_creation(c: &mut Criterion) {
    let sig = signal(42);

    c.bench_function("effect_creation", |b| {
        b.iter(|| {
            black_box(effect(|| {
                let _ = sig.get();
            }));
        });
    });
}

fn bench_effect_propagation(c: &mut Criterion) {
    let sig = signal(0);
    let count = std::rc::Rc::new(std::cell::RefCell::new(0));
    let count_clone = count.clone();

    effect(move || {
        let _ = sig.get();
        *count_clone.borrow_mut() += 1;
    });

    c.bench_function("effect_propagation", |b| {
        let mut i = 0;
        b.iter(|| {
            sig.set(i);
            i += 1;
        });
    });
}

fn bench_batch_updates(c: &mut Criterion) {
    let sig1 = signal(0);
    let sig2 = signal(0);
    let sig3 = signal(0);
    let comp = computed(|| sig1.get() + sig2.get() + sig3.get());

    c.bench_function("batch_updates", |b| {
        let mut i = 0;
        b.iter(|| {
            batch(|| {
                sig1.set(i);
                sig2.set(i + 1);
                sig3.set(i + 2);
            });
            black_box(comp.get());
            i += 1;
        });
    });
}

fn bench_many_signals(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_signals");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let signals: Vec<_> = (0..count).map(|i| signal(i)).collect();
                black_box(signals);
            });
        });
    }

    group.finish();
}

fn bench_many_computed(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_computed");

    for count in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let sig = signal(42);
                let _computed_values: Vec<_> = (0..count)
                    .map(|_i| {
                        let s = sig.clone();
                        computed(move || s.get() * 2)
                    })
                    .collect();
                black_box(_computed_values);
            });
        });
    }

    group.finish();
}

fn bench_diamond_dependency(c: &mut Criterion) {
    // Test the diamond dependency problem: A -> B, A -> C, B -> D, C -> D
    let a = signal(1);
    let b = computed(|| a.get() * 2);
    let c = computed(|| a.get() + 10);
    let d = computed(|| b.get() + c.get());

    c.bench_function("diamond_dependency", |b| {
        let mut i = 0;
        b.iter(|| {
            a.set(i);
            black_box(d.get());
            i += 1;
        });
    });
}

criterion_group!(
    benches,
    bench_signal_creation,
    bench_signal_get,
    bench_signal_set,
    bench_signal_update,
    bench_computed_creation,
    bench_computed_get,
    bench_computed_chain,
    bench_effect_creation,
    bench_effect_propagation,
    bench_batch_updates,
    bench_many_signals,
    bench_many_computed,
    bench_diamond_dependency,
);

criterion_main!(benches);

