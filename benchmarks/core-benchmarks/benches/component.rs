//! Benchmarks for component operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ferric_core::component::{ComponentMetadata, ComponentContext, ViewEncapsulation, ChangeDetectionStrategy};
use ferric_core::reactive::signal;
use ferric_core::di::Injector;
use std::rc::Rc;

fn bench_component_metadata_creation(c: &mut Criterion) {
    c.bench_function("component_metadata_creation", |b| {
        b.iter(|| {
            black_box(ComponentMetadata {
                selector: "test-component".to_string(),
                template: Some("<div>Hello</div>".to_string()),
                styles: vec!["color: red;".to_string()],
                style_urls: vec![],
                encapsulation: ViewEncapsulation::Emulated,
                change_detection: ChangeDetectionStrategy::Default,
                providers: vec![],
            });
        });
    });
}

fn bench_component_context_creation(c: &mut Criterion) {
    let injector = Rc::new(Injector::root());

    c.bench_function("component_context_creation", |b| {
        b.iter(|| {
            black_box(ComponentContext::new(injector.clone()));
        });
    });
}

fn bench_input_bindings(c: &mut Criterion) {
    let mut group = c.benchmark_group("input_bindings");

    for count in [1, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let signals: Vec<_> = (0..count).map(|i| signal(i)).collect();
            b.iter(|| {
                for sig in &signals {
                    sig.set(black_box(42));
                }
            });
        });
    }

    group.finish();
}

fn bench_output_emissions(c: &mut Criterion) {
    use ferric_core::dom::EventEmitter;

    let mut group = c.benchmark_group("output_emissions");

    for count in [1, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let emitters: Vec<_> = (0..count).map(|_| EventEmitter::new()).collect();
            b.iter(|| {
                for emitter in &emitters {
                    emitter.emit(black_box(42));
                }
            });
        });
    }

    group.finish();
}

fn bench_component_lifecycle(c: &mut Criterion) {
    c.bench_function("component_lifecycle", |b| {
        b.iter(|| {
            let count = std::rc::Rc::new(std::cell::RefCell::new(0));
            // Simulate lifecycle
            *count.borrow_mut() += 1; // init
            *count.borrow_mut() -= 1; // destroy
            black_box(count);
        });
    });
}

fn bench_nested_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_components");

    for depth in [1, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                let injector = Rc::new(Injector::root());
                let contexts: Vec<_> = (0..depth)
                    .map(|_| ComponentContext::new(injector.clone()))
                    .collect();
                black_box(contexts);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_component_metadata_creation,
    bench_component_context_creation,
    bench_input_bindings,
    bench_output_emissions,
    bench_component_lifecycle,
    bench_nested_components,
);

criterion_main!(benches);

