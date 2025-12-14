//! Benchmarks for router operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ferric_core::router::{Router, Route, PathMatch};
use ferric_core::reactive::signal;

fn bench_router_creation(c: &mut Criterion) {
    let routes = vec![
        Route {
            path: "/".to_string(),
            component: None,
            children: None,
            path_match: PathMatch::Full,
            redirectTo: None,
            outlet: None,
            can_activate: vec![],
            can_deactivate: vec![],
            can_load: vec![],
            resolve: std::collections::HashMap::new(),
            data: std::collections::HashMap::new(),
            lazy_load: None,
        },
    ];

    c.bench_function("router_creation", |b| {
        b.iter(|| {
            black_box(Router::new(routes.clone()));
        });
    });
}

fn bench_route_matching(c: &mut Criterion) {
    let routes = vec![
        Route {
            path: "/home".to_string(),
            component: None,
            children: None,
            path_match: PathMatch::Full,
            redirectTo: None,
            outlet: None,
            can_activate: vec![],
            can_deactivate: vec![],
            can_load: vec![],
            resolve: std::collections::HashMap::new(),
            data: std::collections::HashMap::new(),
            lazy_load: None,
        },
        Route {
            path: "/about".to_string(),
            component: None,
            children: None,
            path_match: PathMatch::Full,
            redirectTo: None,
            outlet: None,
            can_activate: vec![],
            can_deactivate: vec![],
            can_load: vec![],
            resolve: std::collections::HashMap::new(),
            data: std::collections::HashMap::new(),
            lazy_load: None,
        },
    ];

    let router = Router::new(routes);

    c.bench_function("route_matching", |b| {
        b.iter(|| {
            // Simulate route matching logic
            let path = black_box("/home");
            let matches = path == "/home" || path == "/about";
            black_box(matches);
        });
    });
}

fn bench_param_extraction(c: &mut Criterion) {
    use ferric_core::router::Params;

    c.bench_function("param_extraction", |b| {
        b.iter(|| {
            let mut params = Params::new();
            params.insert("id".to_string(), "123".to_string());
            params.insert("name".to_string(), "test".to_string());
            black_box(params);
        });
    });
}

fn bench_query_params(c: &mut Criterion) {
    use ferric_core::router::QueryParams;

    c.bench_function("query_params", |b| {
        b.iter(|| {
            let mut query = QueryParams::new();
            query.insert("page".to_string(), "1".to_string());
            query.insert("size".to_string(), "10".to_string());
            black_box(query);
        });
    });
}

fn bench_many_routes(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_routes");

    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let routes: Vec<_> = (0..count)
                    .map(|i| Route {
                        path: format!("/route-{}", i),
                        component: None,
                        children: None,
                        path_match: PathMatch::Full,
                        redirectTo: None,
                        outlet: None,
                        can_activate: vec![],
                        can_deactivate: vec![],
                        can_load: vec![],
                        resolve: std::collections::HashMap::new(),
                        data: std::collections::HashMap::new(),
                        lazy_load: None,
                    })
                    .collect();
                let router = Router::new(routes);
                black_box(router);
            });
        });
    }

    group.finish();
}

fn bench_nested_routes(c: &mut Criterion) {
    let routes = vec![
        Route {
            path: "/parent".to_string(),
            component: None,
            children: Some(vec![
                Route {
                    path: "child1".to_string(),
                    component: None,
                    children: None,
                    path_match: PathMatch::Full,
                    redirectTo: None,
                    outlet: None,
                    can_activate: vec![],
                    can_deactivate: vec![],
                    can_load: vec![],
                    resolve: std::collections::HashMap::new(),
                    data: std::collections::HashMap::new(),
                    lazy_load: None,
                },
                Route {
                    path: "child2".to_string(),
                    component: None,
                    children: None,
                    path_match: PathMatch::Full,
                    redirectTo: None,
                    outlet: None,
                    can_activate: vec![],
                    can_deactivate: vec![],
                    can_load: vec![],
                    resolve: std::collections::HashMap::new(),
                    data: std::collections::HashMap::new(),
                    lazy_load: None,
                },
            ]),
            path_match: PathMatch::Prefix,
            redirectTo: None,
            outlet: None,
            can_activate: vec![],
            can_deactivate: vec![],
            can_load: vec![],
            resolve: std::collections::HashMap::new(),
            data: std::collections::HashMap::new(),
            lazy_load: None,
        },
    ];

    c.bench_function("nested_routes", |b| {
        b.iter(|| {
            let router = Router::new(routes.clone());
            black_box(router);
        });
    });
}

criterion_group!(
    benches,
    bench_router_creation,
    bench_route_matching,
    bench_param_extraction,
    bench_query_params,
    bench_many_routes,
    bench_nested_routes,
);

criterion_main!(benches);

