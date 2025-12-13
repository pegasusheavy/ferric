//! # Ferric
//!
//! An Angular-inspired frontend framework built with Rust and WebAssembly.
//!
//! ## Features
//!
//! - **Components**: Reusable UI building blocks with encapsulated logic
//! - **Reactive State**: Fine-grained reactivity with signals and computed values
//! - **Templates**: Declarative view definitions with data binding
//! - **Dependency Injection**: Hierarchical service container
//! - **Routing**: Client-side navigation with guards and resolvers
//! - **Directives**: Extend element behavior with structural and attribute directives
//! - **Lifecycle Hooks**: Component initialization, updates, and cleanup

mod component;
mod di;
mod directives;
mod dom;
mod lifecycle;
mod reactive;
mod router;
mod template;
mod utils;

pub use component::*;
pub use di::{
    Injectable, Injector, InjectionToken, MultiToken, Provider, ProviderBuilder,
    ProviderToken, ResolutionError, Scope, ServiceProvider, Transient,
};
pub use directives::{AttributeDirective, HostBinding};
pub use dom::*;
pub use lifecycle::*;
pub use reactive::*;
pub use router::*;
pub use template::{
    Binding, BindingType, CompiledTemplate, InputBinding, OutputBinding,
    ParseError, TemplateNode, compile_template, parse_template,
    INTERPOLATION_START, INTERPOLATION_END,
};

use wasm_bindgen::prelude::*;

/// Initialize the Ferric framework.
/// Call this function once at application startup.
#[wasm_bindgen(start)]
pub fn init() {
    utils::set_panic_hook();
}

/// Bootstrap a Ferric application with the given root component.
#[wasm_bindgen]
pub fn bootstrap(root_selector: &str) -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("should have a document on window");

    let root = document
        .query_selector(root_selector)
        .map_err(|e| JsValue::from_str(&format!("Failed to query selector: {:?}", e)))?
        .ok_or_else(|| JsValue::from_str(&format!("Root element '{}' not found", root_selector)))?;

    // Initialize the application context
    let app = Application::new(root);
    app.mount()?;

    Ok(())
}

/// The main application container that manages the component tree and services.
#[wasm_bindgen]
pub struct Application {
    root: web_sys::Element,
    injector: di::Injector,
}

#[wasm_bindgen]
impl Application {
    #[wasm_bindgen(constructor)]
    pub fn new(root: web_sys::Element) -> Self {
        Self {
            root,
            injector: di::Injector::root(),
        }
    }

    pub fn mount(&self) -> Result<(), JsValue> {
        // Placeholder for mounting logic
        web_sys::console::log_1(&"Ferric application mounted".into());
        Ok(())
    }
}

