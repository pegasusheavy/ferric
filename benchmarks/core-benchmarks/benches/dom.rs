//! Benchmarks for DOM operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Note: These are placeholder benchmarks as we can't run actual DOM operations in a benchmark context
// In a real scenario, you'd use wasm-bindgen-test with a headless browser

fn bench_dom_creation(c: &mut Criterion) {
    c.bench_function("dom_element_creation_concept", |b| {
        b.iter(|| {
            // Simulate the cost of creating a DOM element
            let element_data = black_box(("div", vec![("class", "test")]));
            black_box(element_data);
        });
    });
}

fn bench_attribute_setting(c: &mut Criterion) {
    let mut group = c.benchmark_group("attribute_setting");

    for count in [1, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let attributes: Vec<_> = (0..count)
                    .map(|i| (format!("attr-{}", i), format!("value-{}", i)))
                    .collect();
                black_box(attributes);
            });
        });
    }

    group.finish();
}

fn bench_text_content_update(c: &mut Criterion) {
    c.bench_function("text_content_update", |b| {
        let mut text = String::from("Initial text");
        b.iter(|| {
            text.clear();
            text.push_str(black_box("Updated text"));
        });
    });
}

fn bench_class_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_list_operations");

    for count in [1, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let classes: Vec<_> = (0..count)
                    .map(|i| format!("class-{}", i))
                    .collect();
                black_box(classes);
            });
        });
    }

    group.finish();
}

fn bench_style_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("style_updates");

    for count in [1, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let styles: Vec<_> = (0..count)
                    .map(|i| (format!("prop-{}", i), format!("value-{}", i)))
                    .collect();
                black_box(styles);
            });
        });
    }

    group.finish();
}

fn bench_event_listener_registration(c: &mut Criterion) {
    use ferric_core::dom::EventEmitter;

    c.bench_function("event_listener_registration", |b| {
        b.iter(|| {
            let emitter = EventEmitter::<i32>::new();
            emitter.subscribe(|_value| {
                // Handler
            });
            black_box(emitter);
        });
    });
}

fn bench_virtual_dom_diffing(c: &mut Criterion) {
    // Simulate diffing algorithm
    #[derive(Clone, PartialEq)]
    struct VNode {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<VNode>,
    }

    let old_tree = VNode {
        tag: "div".to_string(),
        attrs: vec![("class".to_string(), "container".to_string())],
        children: vec![
            VNode {
                tag: "span".to_string(),
                attrs: vec![],
                children: vec![],
            },
        ],
    };

    let new_tree = VNode {
        tag: "div".to_string(),
        attrs: vec![("class".to_string(), "container-updated".to_string())],
        children: vec![
            VNode {
                tag: "span".to_string(),
                attrs: vec![],
                children: vec![],
            },
            VNode {
                tag: "span".to_string(),
                attrs: vec![],
                children: vec![],
            },
        ],
    };

    c.bench_function("virtual_dom_diffing", |b| {
        b.iter(|| {
            let has_attr_change = old_tree.attrs != new_tree.attrs;
            let has_children_change = old_tree.children.len() != new_tree.children.len();
            black_box((has_attr_change, has_children_change));
        });
    });
}

criterion_group!(
    benches,
    bench_dom_creation,
    bench_attribute_setting,
    bench_text_content_update,
    bench_class_list_operations,
    bench_style_updates,
    bench_event_listener_registration,
    bench_virtual_dom_diffing,
);

criterion_main!(benches);

