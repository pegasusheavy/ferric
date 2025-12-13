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
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric::prelude::*;
//!
//! #[component(
//!     selector = "app-counter",
//!     template = r#"
//!         <div>
//!             <span>Count: {{ count }}</span>
//!             <button (click)="increment()">+</button>
//!         </div>
//!     "#
//! )]
//! pub struct CounterComponent {
//!     count: Signal<i32>,
//! }
//!
//! impl CounterComponent {
//!     fn increment(&self) {
//!         self.count.update(|n| n + 1);
//!     }
//! }
//! ```

pub mod change_detection;
pub mod component;
pub mod content;
pub mod devtools;
pub mod di;
pub mod directives;
pub mod dom;
pub mod hydration;
pub mod i18n;
pub mod lifecycle;
pub mod logging;
pub mod pipes;
pub mod reactive;
pub mod router;
pub mod template;
pub mod testing;
mod utils;

// Re-export macros from ferric_macros
pub use ferric_macros::{
    component,
    injectable,
    input,
    output,
    host_listener,
    host_binding,
    view_child,
    view_children,
    content_child,
    content_children,
    Component,
    Injectable,
};

// Re-export commonly used types
pub use component::{
    Component as ComponentTrait, ComponentMetadata, ComponentContext,
    // View Encapsulation
    ViewEncapsulation, ViewEncapsulator, StyleEncapsulator, TemplateEncapsulator,
    EncapsulationConfig, EncapsulatedComponent,
    generate_component_id, get_or_encapsulate_styles,
    CONTENT_ATTR_PREFIX, HOST_ATTR_PREFIX,
};
pub use di::{
    // Core types
    Injectable as InjectableTrait, Injector, ResolutionError,
    // Providers
    Provider, ProviderBuilder, ServiceProvider,
    // Tokens
    InjectionToken, MultiToken, ProviderToken, FactoryToken, TokenTypeId,
    // Scopes
    Scope, ProvidedIn, ProvideConfig, SelfProviding, Transient,
    // Resolution modifiers
    Optional, Self_, SkipSelf, Host, Resolve, ResolutionOptions,
};
pub use directives::{
    AttributeDirective, HostBinding, StructuralDirective,
    // Switch directive
    FeSwitchContainer, SwitchCase, SwitchValue,
    ParsedSwitch, SwitchBlock, ParsedCase,
    parse_switch_template, transform_switch_template,
};
pub use dom::{EventEmitter, EventListener};
pub use hydration::{
    FerricHydration, Hydratable, HydrationContext, HydrationManager,
    init_hydration_api,
};
pub use pipes::{
    // Core types
    Pipe, PipeArgs, PipeValue, ParsedPipe,
    // Registry
    PipeRegistry, register_pipe, get_pipe,
    // Built-in pipes
    UppercasePipe, LowercasePipe, TitlecasePipe, TrimPipe, SlicePipe,
    NumberPipe, CurrencyPipe, PercentPipe,
    DatePipe,
    JsonPipe, DefaultPipe, ReplacePipe, PadStartPipe, PadEndPipe,
    TruncatePipe, ReversePipe, RepeatPipe, JoinPipe, SplitPipe, KeyvaluePipe,
    // Async pipe
    AsyncPipe, AsyncState, AsyncValue, AsyncPipeRegistry,
    async_pending, async_resolved, async_error,
    // Functions
    apply_pipe, apply_pipes, parse_pipe_expression, parse_piped_expression,
};
pub use lifecycle::{
    // Core traits
    Lifecycle, Changes, Change, TypedChange,
    // State
    LifecycleState,
    // Manager
    LifecycleManager, LifecycleError, LifecycleResult, ComponentId,
    lifecycle_manager, register_component, get_component_state,
    // Cleanup
    CleanupRegistry, CleanupHandle, CleanupGuard, on_cleanup,
    set_cleanup_context, clear_cleanup_context, with_cleanup_context,
    // Extended lifecycle traits
    AsyncLifecycle, ErrorBoundary, NavigationLifecycle,
    VisibilityLifecycle, FocusLifecycle, ResizeLifecycle, DocumentLifecycle,
    HookError, HookResult, LifecycleHookBuilder,
    // Marker traits
    OnInit, OnDestroy, OnChanges, DoCheck,
    AfterViewInit, AfterContentInit, AfterViewChecked, AfterContentChecked,
};
pub use content::{
    // Core projection types
    ContentProjection, ProjectedContent, ProjectionContext,
    project_content, create_projection_context,
    // Selectors
    ContentSelector, SelectorKind, matches_selector,
    // Slots
    ContentSlot, SlotRef, find_slots, replace_slots,
};
pub use reactive::{
    // Core primitives
    Signal, WritableSignal, ReadonlySignal, Computed, Effect,
    signal, computed, effect,
    // Utilities
    batch, untracked, derived, watch, watch_immediate,
    // Resources
    Resource, ResourceState, create_resource, create_resource_once,
    // Traits and IDs
    Reactive, SubscriptionId, ReactiveId,
};
pub use router::{
    Router, Route, Routes, ActivatedRoute, GuardResult,
    // Lazy loading
    LazyRouteModule, LazyModuleRegistry, LazyLoadError, LazyLoadResult, LoadState,
    PreloadConfig, PreloadStrategy, lazy, lazy_factory, lazy_async,
    // Route configuration
    RunGuardsAndResolvers, PathMatch,
};
pub use change_detection::{
    // Core types
    ApplicationRef, ApplicationConfig, ChangeDetectorRef, ChangeDetectorStatus,
    DetectorTree, DetectorId, ChangeDetectionResult, ChangeDetectionContext,
    // Scheduler
    Scheduler, SchedulerConfig, schedule, schedule_on_stable,
    // Traits
    ChangeDetectable,
};
pub use template::{
    // Core types
    Binding, BindingType, CompiledTemplate, InputBinding, OutputBinding,
    ParseError, TemplateNode, compile_template, parse_template,
    // Template context and rendering
    TemplateContext, ContextBuilder, TemplateInstance, TemplateRenderer,
    TemplateBuilder, render_template,
    // Expression evaluation
    ExprValue, evaluate, evaluate_to_string, evaluate_to_bool,
};
pub use logging::{
    // Core types
    Level, Logger, LogConfig, Record, RecordBuilder,
    // Configuration
    configure, set_log_level, log_level, set_target_level, enabled,
    // Output
    ConsoleOutput, LogOutput,
};
pub use i18n::{
    // Service
    I18nService,
    // Locale
    Locale, LocaleInfo, parse_locale, get_browser_locale,
    // Translation
    TranslationStore, t, tp, translate, translate_plural,
    // Pluralization
    PluralCategory, get_plural_category,
    // Formatting
    DateFormat, NumberFormat, format_number, format_currency, format_date,
};

// Note: Logging macros (trace!, debug!, info!, warn!, error!, log_enabled!)
// are automatically exported at the crate root by #[macro_export].
// Import them via: use ferric::{trace, debug, info, warn, error};

use wasm_bindgen::prelude::*;

/// Prelude module - import everything you need with `use ferric::prelude::*`
pub mod prelude {
    // Macros
    pub use ferric_macros::{
        component,
        injectable,
        input,
        output,
        host_listener,
        host_binding,
        view_child,
        view_children,
        content_child,
        content_children,
        Component,
        Injectable,
    };

    // Core traits
    pub use crate::component::Component as ComponentTrait;
    pub use crate::di::Injectable as InjectableTrait;
    pub use crate::lifecycle::{
        Lifecycle, Changes, Change, LifecycleState,
        on_cleanup, CleanupGuard, LifecycleHookBuilder,
    };
    pub use crate::reactive::Reactive;
    pub use crate::directives::AttributeDirective;

    // Reactive primitives
    pub use crate::reactive::{
        Signal, Computed, Effect, signal, computed, effect,
        batch, untracked, derived, watch, watch_immediate,
        Resource, ResourceState, create_resource,
        ReactiveId, SubscriptionId,
    };

    // DI
    pub use crate::di::{
        Injector, Scope, InjectionToken, MultiToken,
        Optional, Self_, SkipSelf, Resolve,
        ProvidedIn, ProvideConfig,
    };

    // Component
    pub use crate::component::{
        ComponentMetadata, ViewEncapsulation, ChangeDetectionStrategy,
        ViewEncapsulator, EncapsulatedComponent,
    };

    // Content Projection
    pub use crate::content::{
        ContentProjection, ProjectedContent, ProjectionContext,
        ContentSelector, ContentSlot, project_content,
    };

    // Change Detection
    pub use crate::change_detection::{
        ApplicationRef, ChangeDetectorRef, ChangeDetectorStatus,
        Scheduler, schedule, schedule_on_stable,
    };

    // DOM utilities
    pub use crate::dom::{EventEmitter, EventListener, document, window};

    // Hydration
    pub use crate::hydration::{
        Hydratable, HydrationContext, HydrationManager, init_hydration_api,
    };

    // Router
    pub use crate::router::{Router, Route, Routes, ActivatedRoute};

    // Template
    pub use crate::template::{
        Binding, BindingType, CompiledTemplate, TemplateContext,
        TemplateInstance, TemplateBuilder, compile_template, render_template,
        evaluate, evaluate_to_string, evaluate_to_bool,
    };

    // Logging
    pub use crate::logging::{Level, Logger, LogConfig, configure, set_log_level};
    // Logging macros: trace!, debug!, info!, warn!, error!, log_enabled!
    // are automatically available via `use ferric::*` or `use ferric::prelude::*`

    // i18n
    pub use crate::i18n::{
        I18nService, Locale, parse_locale, get_browser_locale,
        t, tp, translate, translate_plural,
        PluralCategory, DateFormat, NumberFormat,
    };

    // Pipes
    pub use crate::pipes::{
        Pipe, PipeArgs, PipeValue,
        PipeRegistry, register_pipe, get_pipe,
        apply_pipe, apply_pipes, parse_pipe_expression,
    };

    // Re-export std::rc::Rc for convenience
    pub use std::rc::Rc;
}

/// Initialize the Ferric framework.
///
/// Call this function once at application startup to set up panic hooks
/// and other framework initialization. This is automatically called by
/// most applications via `console_error_panic_hook::set_once()`.
///
/// Note: This is NOT marked as `#[wasm_bindgen(start)]` to avoid conflicts
/// when applications define their own start functions.
#[wasm_bindgen]
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
    #[allow(dead_code)]
    root: web_sys::Element,
    #[allow(dead_code)]
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
        info!(target: "ferric", "Ferric application mounted");
        Ok(())
    }
}
