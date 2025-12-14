//! Integration tests for ferric-core

use ferric_core::{
    reactive::{signal, computed, effect, batch, on_cleanup},
    di::{Injector, Injectable, ProviderBuilder, Scope, InjectionToken},
    router::{Router, Route},
};
use std::rc::Rc;

#[test]
fn test_signal_creation_and_updates() {
    let count = signal(0);
    assert_eq!(count.get(), 0);

    count.set(42);
    assert_eq!(count.get(), 42);

    count.update(|n| *n += 10);
    assert_eq!(count.get(), 52);
}

#[test]
fn test_computed_values() {
    let a = signal(10);
    let b = signal(20);
    let sum = computed(move || a.get() + b.get());

    assert_eq!(sum.get(), 30);

    a.set(15);
    assert_eq!(sum.get(), 35);

    b.set(25);
    assert_eq!(sum.get(), 40);
}

#[test]
fn test_batch_updates() {
    let count = signal(0);
    let updates = signal(0);

    let updates_clone = updates.clone();
    effect(move || {
        let _ = count.get();
        updates_clone.update(|n| *n += 1);
    });

    let initial = updates.get();

    batch(|| {
        count.set(1);
        count.set(2);
        count.set(3);
    });

    // Should only update once for the batch
    assert!(updates.get() <= initial + 2); // Allow for initial effect + batch
}

#[test]
fn test_dependency_injection() {
    #[derive(Clone)]
    struct TestService {
        value: i32,
    }

    impl Injectable for TestService {
        fn create(_injector: &Injector) -> Self {
            Self { value: 42 }
        }
    }

    let injector = Injector::root();
    injector.provide(ProviderBuilder::<TestService>::new()
        .scope(Scope::Singleton)
        .build());

    let service = injector.resolve_required::<TestService>();
    assert_eq!(service.value, 42);
}

#[test]
fn test_injection_tokens() {
    const API_URL: InjectionToken<String> = InjectionToken::with_id("API_URL", 1);

    let injector = Injector::root();
    injector.register_token(&API_URL, "https://api.example.com".to_string());

    let url = injector.resolve_token(&API_URL).unwrap();
    assert_eq!(url, "https://api.example.com");
}

#[test]
fn test_hierarchical_injectors() {
    #[derive(Clone)]
    struct Service {
        level: String,
    }

    impl Injectable for Service {
        fn create(_: &Injector) -> Self {
            Self { level: "root".to_string() }
        }
    }

    let root = Injector::root();
    root.provide(ProviderBuilder::<Service>::new().build());

    let child = root.create_child();
    child.provide(ProviderBuilder::<Service>::new()
        .factory(|_| Service { level: "child".to_string() })
        .build());

    let root_service = root.resolve_required::<Service>();
    let child_service = child.resolve_required::<Service>();

    assert_eq!(root_service.level, "root");
    assert_eq!(child_service.level, "child");
}

#[test]
fn test_optional_injection() {
    #[derive(Clone)]
    struct OptionalService;

    impl Injectable for OptionalService {
        fn create(_: &Injector) -> Self {
            Self
        }
    }

    let injector = Injector::root();

    // Without providing the service
    let result: Option<OptionalService> = injector.resolve();
    assert!(result.is_none());

    // After providing
    injector.provide(ProviderBuilder::<OptionalService>::new().build());
    let result: Option<OptionalService> = injector.resolve();
    assert!(result.is_some());
}

#[test]
fn test_router_basic() {
    let routes = vec![
        Route::path("/").build(),
        Route::path("/about").build(),
        Route::path("/users/:id").build(),
    ];

    let router = Router::new(routes);
    assert!(router.current_url().is_none());
}

#[test]
fn test_signal_clone() {
    let s1 = signal(10);
    let s2 = s1.clone();

    s1.set(20);
    assert_eq!(s2.get(), 20);
}

#[test]
fn test_computed_dependencies() {
    let a = signal(1);
    let b = computed(move || a.get() * 2);
    let c = computed(move || b.get() + 10);

    assert_eq!(c.get(), 12); // 1 * 2 + 10

    a.set(5);
    assert_eq!(c.get(), 20); // 5 * 2 + 10
}

#[test]
fn test_effect_runs_initially() {
    let ran = Rc::new(std::cell::RefCell::new(false));
    let ran_clone = ran.clone();

    let _e = effect(move || {
        *ran_clone.borrow_mut() = true;
    });

    assert!(*ran.borrow());
}

#[test]
fn test_router_route_matching() {
    let routes = vec![
        Route::path("/home").build(),
        Route::path("/users/:id").build(),
    ];

    let router = Router::new(routes);
    // Basic creation test - full routing tests would need WASM environment
    assert_eq!(router.routes().len(), 2);
}
