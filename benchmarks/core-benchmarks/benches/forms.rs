//! Benchmarks for forms operations.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use ferric_forms::controls::{FormControl, FormGroup, FormArray};
use std::rc::Rc;

fn bench_form_control_creation(c: &mut Criterion) {
    c.bench_function("form_control_creation", |b| {
        b.iter(|| {
            black_box(FormControl::new("test value".to_string()));
        });
    });
}

fn bench_form_control_set_value(c: &mut Criterion) {
    let control = FormControl::new(String::new());

    c.bench_function("form_control_set_value", |b| {
        let mut i = 0;
        b.iter(|| {
            control.set_value(format!("value-{}", i));
            i += 1;
        });
    });
}

fn bench_form_control_validation(c: &mut Criterion) {
    let control = FormControl::new(String::new());

    c.bench_function("form_control_validation", |b| {
        b.iter(|| {
            control.set_value(black_box("test".to_string()));
            black_box(control.is_valid());
        });
    });
}

fn bench_form_group_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("form_group_creation");

    for count in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut controls = std::collections::HashMap::new();
                for i in 0..count {
                    controls.insert(
                        format!("field-{}", i),
                        Rc::new(FormControl::new(String::new())) as Rc<dyn ferric_forms::controls::AbstractControl>,
                    );
                }
                black_box(FormGroup::new(controls));
            });
        });
    }

    group.finish();
}

fn bench_form_group_get_value(c: &mut Criterion) {
    let mut controls = std::collections::HashMap::new();
    for i in 0..10 {
        controls.insert(
            format!("field-{}", i),
            Rc::new(FormControl::new(format!("value-{}", i))) as Rc<dyn ferric_forms::controls::AbstractControl>,
        );
    }
    let group = FormGroup::new(controls);

    c.bench_function("form_group_get_value", |b| {
        b.iter(|| {
            black_box(group.get_value());
        });
    });
}

fn bench_form_group_validation(c: &mut Criterion) {
    let mut controls = std::collections::HashMap::new();
    for i in 0..10 {
        let control = FormControl::new(String::new());
        controls.insert(
            format!("field-{}", i),
            Rc::new(control) as Rc<dyn ferric_forms::controls::AbstractControl>,
        );
    }
    let group = FormGroup::new(controls);

    c.bench_function("form_group_validation", |b| {
        b.iter(|| {
            black_box(group.is_valid());
        });
    });
}

fn bench_form_array_operations(c: &mut Criterion) {
    c.bench_function("form_array_push", |b| {
        b.iter(|| {
            let array = FormArray::new();
            array.push(Rc::new(FormControl::new(String::new())));
            black_box(array);
        });
    });
}

fn bench_nested_form_groups(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_form_groups");

    for depth in [2, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                let mut current: Rc<dyn ferric_forms::controls::AbstractControl> =
                    Rc::new(FormControl::new("leaf".to_string()));

                for _ in 0..depth {
                    let mut controls = std::collections::HashMap::new();
                    controls.insert("child".to_string(), current);
                    current = Rc::new(FormGroup::new(controls));
                }

                black_box(current);
            });
        });
    }

    group.finish();
}

fn bench_validators(c: &mut Criterion) {
    c.bench_function("validator_simulation", |b| {
        b.iter(|| {
            // Simulate validation logic
            let value = black_box("test@example.com");
            let is_valid = value.contains('@') && value.contains('.');
            black_box(is_valid);
        });
    });
}

criterion_group!(
    benches,
    bench_form_control_creation,
    bench_form_control_set_value,
    bench_form_control_validation,
    bench_form_group_creation,
    bench_form_group_get_value,
    bench_form_group_validation,
    bench_form_array_operations,
    bench_nested_form_groups,
    bench_validators,
);

criterion_main!(benches);

