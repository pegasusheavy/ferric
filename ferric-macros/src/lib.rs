//! # Ferric Macros
//!
//! Procedural macros for the Ferric framework, providing Angular-like
//! decorators for building web applications.
//!
//! ## Available Macros
//!
//! ### Component Definition
//! - `#[component]` - Define a component with selector, template, and styles
//! - `#[derive(Component)]` - Auto-implement the Component trait
//!
//! ### Module Organization
//! - `#[module]` - Define a module (like @NgModule)
//!
//! ### Directives & Pipes
//! - `#[directive]` - Create a directive (like @Directive)
//! - `#[pipe]` - Create a pipe/transform (like @Pipe)
//!
//! ### Dependency Injection
//! - `#[injectable]` - Mark a service as injectable
//! - `#[derive(Injectable)]` - Auto-implement the Injectable trait
//!
//! ### Component IO
//! - `#[input]` - Mark a field as an input property
//! - `#[output]` - Mark a field as an output event emitter
//!
//! ### Host Bindings
//! - `#[host_listener]` - Listen to host element events
//! - `#[host_binding]` - Bind to host element properties
//!
//! ### View Queries
//! - `#[view_child]` - Query a single child element
//! - `#[view_children]` - Query multiple child elements
//!
//! ### Routing
//! - `#[guard]` - Define route guards (CanActivate, CanDeactivate, etc.)
//! - `#[resolver]` - Define route data resolvers
//!
//! ### HTTP
//! - `#[interceptor]` - Define HTTP interceptors
//!
//! ## Example
//!
//! ```ignore
//! use ferric_core::prelude::*;
//!
//! // Component with inputs and outputs
//! #[component(
//!     selector = "app-counter",
//!     template = r#"
//!         <div class="counter">
//!             <span>{{ count }}</span>
//!             <button (click)="increment()">+</button>
//!         </div>
//!     "#
//! )]
//! pub struct CounterComponent {
//!     #[input]
//!     initial: i32,
//!
//!     #[output]
//!     count_changed: EventEmitter<i32>,
//!
//!     count: Signal<i32>,
//! }
//!
//! // Directive
//! #[directive(selector = "[appHighlight]")]
//! pub struct HighlightDirective {
//!     #[input]
//!     color: String,
//! }
//!
//! // Pipe
//! #[pipe(name = "uppercase")]
//! pub struct UpperCasePipe;
//!
//! // Guard
//! #[guard(type = "can_activate")]
//! pub struct AuthGuard;
//!
//! // Module
//! #[module(
//!     declarations = vec![CounterComponent::declaration()],
//!     providers = vec![AuthService::provider()]
//! )]
//! pub struct AppModule;
//! ```

mod component;
mod component_file;
mod injectable;
mod input_output;
mod host;
mod view_query;
mod utils;

// New utility macros
mod reactive_macros;
mod template_macros;
mod event_macros;
mod di_macros;
mod test_macros;

// Angular-like decorator macros
mod module_macro;
mod directive_macro;
mod pipe_macro;
mod guard_macro;
mod interceptor_macro;
mod resolver_macro;

use proc_macro::TokenStream;

/// Define a component with Angular-like metadata.
///
/// ## Attributes
///
/// - `selector` (required) - CSS selector for the component (e.g., "app-root")
/// - `template` - Inline template string
/// - `template_url` - Path to external template file
/// - `styles` - Inline styles as a string or array of strings
/// - `style_urls` - Paths to external stylesheets
/// - `encapsulation` - View encapsulation mode: `Emulated`, `ShadowDom`, or `None`
/// - `change_detection` - Change detection strategy: `Default` or `OnPush`
///
/// ## Example
///
/// ```ignore
/// #[component(
///     selector = "app-hello",
///     template = "<h1>Hello, {{ name }}!</h1>",
///     styles = "h1 { color: blue; }"
/// )]
/// pub struct HelloComponent {
///     name: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component::component_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derive the `Component` trait for a struct.
///
/// This derive macro requires the `#[component]` attribute to be present
/// with at least a `selector` specified.
#[proc_macro_derive(Component, attributes(component, input, output))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    component::derive_component_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Mark a service as injectable for dependency injection.
///
/// ## Attributes
///
/// - `provided_in` - Where to provide the service:
///   - `"root"` - Singleton at the application level (default)
///   - `"any"` - New instance for each injector
///   - `"platform"` - Shared across applications
///
/// ## Example
///
/// ```ignore
/// #[injectable(provided_in = "root")]
/// pub struct UserService {
///     users: Vec<User>,
/// }
/// ```
#[proc_macro_attribute]
pub fn injectable(args: TokenStream, input: TokenStream) -> TokenStream {
    injectable::injectable_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derive the `Injectable` trait for a struct.
///
/// The struct must have a `new()` method or all fields must implement `Default`,
/// or you can specify dependencies using `#[inject]` attributes.
///
/// ## Example
///
/// ```ignore
/// #[derive(Injectable)]
/// pub struct MyService {
///     #[inject]
///     http: HttpClient,
///
///     #[inject]
///     config: AppConfig,
/// }
/// ```
#[proc_macro_derive(Injectable, attributes(inject))]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    injectable::derive_injectable_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Mark a field as an input property that can receive values from parent components.
///
/// ## Attributes
///
/// - `alias` - Alternative name for the input binding
/// - `required` - Whether this input must be provided (default: false)
/// - `transform` - Function to transform the input value
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-user")]
/// pub struct UserComponent {
///     #[input]
///     name: String,
///
///     #[input(alias = "userId", required)]
///     id: u32,
///
///     #[input(transform = parse_date)]
///     birthday: Date,
/// }
/// ```
#[proc_macro_attribute]
pub fn input(args: TokenStream, input: TokenStream) -> TokenStream {
    input_output::input_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Mark a field as an output event emitter.
///
/// Output fields should be of type `EventEmitter<T>`.
///
/// ## Attributes
///
/// - `alias` - Alternative name for the output binding
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-button")]
/// pub struct ButtonComponent {
///     #[output]
///     clicked: EventEmitter<()>,
///
///     #[output(alias = "valueChange")]
///     value_changed: EventEmitter<String>,
/// }
/// ```
#[proc_macro_attribute]
pub fn output(args: TokenStream, input: TokenStream) -> TokenStream {
    input_output::output_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Listen to events on the host element.
///
/// ## Arguments
///
/// - Event name (required) - The DOM event to listen for
/// - `$event` - Pass the event object to the handler
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-clickable")]
/// impl ClickableComponent {
///     #[host_listener("click", "$event")]
///     fn on_click(&self, event: MouseEvent) {
///         // Handle click
///     }
///
///     #[host_listener("keydown.enter")]
///     fn on_enter(&self) {
///         // Handle enter key
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn host_listener(args: TokenStream, input: TokenStream) -> TokenStream {
    host::host_listener_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Bind a property or attribute to the host element.
///
/// ## Syntax
///
/// - `#[host_binding("class.active")]` - Bind to a CSS class
/// - `#[host_binding("style.color")]` - Bind to a style property
/// - `#[host_binding("attr.aria-label")]` - Bind to an attribute
/// - `#[host_binding("disabled")]` - Bind to a property
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-button")]
/// pub struct ButtonComponent {
///     #[host_binding("class.active")]
///     is_active: bool,
///
///     #[host_binding("style.backgroundColor")]
///     bg_color: String,
///
///     #[host_binding("attr.aria-disabled")]
///     aria_disabled: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn host_binding(args: TokenStream, input: TokenStream) -> TokenStream {
    host::host_binding_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Query for a single child element in the component's view.
///
/// ## Arguments
///
/// - Selector string - CSS selector or template reference
/// - `read` - What to read from the element (optional)
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-form")]
/// pub struct FormComponent {
///     #[view_child("input.name")]
///     name_input: Option<ElementRef>,
///
///     #[view_child("#submitBtn", read = ElementRef)]
///     submit_button: Option<ElementRef>,
/// }
/// ```
#[proc_macro_attribute]
pub fn view_child(args: TokenStream, input: TokenStream) -> TokenStream {
    view_query::view_child_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Query for multiple child elements in the component's view.
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-list")]
/// pub struct ListComponent {
///     #[view_children("li.item")]
///     items: Vec<ElementRef>,
/// }
/// ```
#[proc_macro_attribute]
pub fn view_children(args: TokenStream, input: TokenStream) -> TokenStream {
    view_query::view_children_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Query for a single projected content element.
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-card")]
/// pub struct CardComponent {
///     #[content_child("header")]
///     header: Option<ElementRef>,
/// }
/// ```
#[proc_macro_attribute]
pub fn content_child(args: TokenStream, input: TokenStream) -> TokenStream {
    view_query::content_child_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Query for multiple projected content elements.
///
/// ## Example
///
/// ```ignore
/// #[component(selector = "app-tabs")]
/// pub struct TabsComponent {
///     #[content_children("app-tab")]
///     tabs: Vec<TabComponent>,
/// }
/// ```
#[proc_macro_attribute]
pub fn content_children(args: TokenStream, input: TokenStream) -> TokenStream {
    view_query::content_children_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Include an HTML template file at compile time.
///
/// This is a convenience wrapper around `include_str!()` with better
/// error messages for template files.
///
/// ## Example
///
/// ```ignore
/// const TEMPLATE: &str = include_template!("my_component.html");
/// ```
#[proc_macro]
pub fn include_template(input: TokenStream) -> TokenStream {
    component_file::expand_include_template(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Include a CSS styles file at compile time.
///
/// This is a convenience wrapper around `include_str!()` with better
/// error messages for style files.
///
/// ## Example
///
/// ```ignore
/// const STYLES: &str = include_styles!("my_component.css");
/// ```
#[proc_macro]
pub fn include_styles(input: TokenStream) -> TokenStream {
    component_file::expand_include_styles(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Define TEMPLATE and STYLES constants from external files.
///
/// This macro creates two constants: `TEMPLATE` and `STYLES` from the
/// specified files. It's similar to Angular's `templateUrl` and `styleUrls`.
///
/// ## Example
///
/// ```ignore
/// // With both template and styles
/// component_files!("my_component.html", "my_component.css");
///
/// // Template only
/// component_files!("my_component.html");
///
/// // Then use them in your component:
/// fn render(&self) -> Element {
///     inject_styles("my-component", STYLES);
///     html(TEMPLATE)
/// }
/// ```
#[proc_macro]
pub fn component_files(input: TokenStream) -> TokenStream {
    component_file::expand_component_files(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Reactive Programming Macros
// ============================================================================

/// Create a computed signal with concise syntax.
///
/// ## Example
///
/// ```ignore
/// let count = signal(0);
/// let doubled = computed!(|| count.get() * 2);
/// ```
#[proc_macro]
pub fn computed(input: TokenStream) -> TokenStream {
    reactive_macros::computed_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create an effect with concise syntax.
///
/// ## Example
///
/// ```ignore
/// let count = signal(0);
/// effect!(|| {
///     println!("Count is: {}", count.get());
/// });
/// ```
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    reactive_macros::effect_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a signal with concise syntax.
///
/// ## Example
///
/// ```ignore
/// let count = signal!(0);
/// let name = signal!("Alice".to_string());
/// ```
#[proc_macro]
pub fn signal(input: TokenStream) -> TokenStream {
    reactive_macros::signal_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Batch multiple signal updates.
///
/// ## Example
///
/// ```ignore
/// batch!(|| {
///     count.set(1);
///     name.set("Bob".to_string());
/// });
/// ```
#[proc_macro]
pub fn batch(input: TokenStream) -> TokenStream {
    reactive_macros::batch_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a memoized value (cached computation).
///
/// ## Example
///
/// ```ignore
/// let expensive_value = memo!(|| {
///     // Expensive computation
///     compute_fibonacci(50)
/// });
/// ```
#[proc_macro]
pub fn memo(input: TokenStream) -> TokenStream {
    reactive_macros::memo_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Watch a value and react to changes (alias for effect).
///
/// ## Example
///
/// ```ignore
/// watch!(|| {
///     if count.get() > 10 {
///         alert("Count exceeded 10!");
///     }
/// });
/// ```
#[proc_macro]
pub fn watch(input: TokenStream) -> TokenStream {
    reactive_macros::watch_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a lazy-initialized static value.
///
/// ## Example
///
/// ```ignore
/// let config = lazy!(|| load_config());
/// ```
#[proc_macro]
pub fn lazy(input: TokenStream) -> TokenStream {
    reactive_macros::lazy_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Template Macros
// ============================================================================

/// Parse HTML template at compile time.
///
/// ## Example
///
/// ```ignore
/// let template = html!(r#"
///     <div class="container">
///         <h1>{{ title }}</h1>
///     </div>
/// "#);
/// ```
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    template_macros::html_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Define CSS with validation.
///
/// ## Example
///
/// ```ignore
/// let styles = css!(r#"
///     .container { padding: 20px; }
///     h1 { color: blue; }
/// "#);
/// ```
#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    template_macros::css_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Validate and create a component selector.
///
/// ## Example
///
/// ```ignore
/// let selector = selector!("app-my-component");
/// ```
#[proc_macro]
pub fn selector(input: TokenStream) -> TokenStream {
    template_macros::selector_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Event Handling Macros
// ============================================================================

/// Create an event handler with concise syntax.
///
/// ## Example
///
/// ```ignore
/// let handler = on!(click => |event| {
///     println!("Clicked!");
/// });
/// ```
#[proc_macro]
pub fn on(input: TokenStream) -> TokenStream {
    event_macros::on_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Emit an event from an EventEmitter.
///
/// ## Example
///
/// ```ignore
/// emit!(self.count_changed, count);
/// ```
#[proc_macro]
pub fn emit(input: TokenStream) -> TokenStream {
    event_macros::emit_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Dependency Injection Macros
// ============================================================================

/// Inject a dependency from the current injector.
///
/// ## Example
///
/// ```ignore
/// let user_service = inject!(UserService);
/// ```
#[proc_macro]
pub fn inject(input: TokenStream) -> TokenStream {
    di_macros::inject_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Register a provider in the current injector.
///
/// ## Example
///
/// ```ignore
/// provide!(UserService);
/// ```
#[proc_macro]
pub fn provide(input: TokenStream) -> TokenStream {
    di_macros::provide_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Testing Macros
// ============================================================================

/// Create a component test with automatic TestBed setup.
///
/// ## Example
///
/// ```ignore
/// component_test! {
///     fn test_counter() {
///         let fixture = test_bed.create_component::<CounterComponent>();
///         assert_eq!(fixture.instance().count.get(), 0);
///     }
/// }
/// ```
#[proc_macro]
pub fn component_test(input: TokenStream) -> TokenStream {
    test_macros::component_test_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create an async test.
///
/// ## Example
///
/// ```ignore
/// async_test! {
///     fn test_fetch_users() {
///         let users = fetch_users().await;
///         assert!(!users.is_empty());
///     }
/// }
/// ```
#[proc_macro]
pub fn async_test(input: TokenStream) -> TokenStream {
    test_macros::async_test_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Angular-like Decorator Macros
// ============================================================================

/// Define a module (Angular @NgModule equivalent).
///
/// Modules organize an application into cohesive blocks of functionality.
///
/// ## Attributes
///
/// - `imports` - Other modules to import
/// - `providers` - Services to provide
/// - `declarations` - Components, directives, and pipes
/// - `exports` - Public components/directives
/// - `bootstrap` - Root component to bootstrap
///
/// ## Example
///
/// ```ignore
/// #[module(
///     imports = vec![CommonModule, FormsModule],
///     providers = vec![UserService::provider()],
///     declarations = vec![HeaderComponent, FooterComponent],
///     exports = vec!["header", "footer"],
///     bootstrap = "app-root"
/// )]
/// pub struct AppModule;
/// ```
#[proc_macro_attribute]
pub fn module(args: TokenStream, input: TokenStream) -> TokenStream {
    module_macro::module_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Define a directive (Angular @Directive equivalent).
///
/// Directives add behavior to elements without a template.
///
/// ## Attributes
///
/// - `selector` (required) - CSS selector for the directive
/// - `standalone` - Whether this is a standalone directive
/// - `host` - Host element bindings
/// - `export_as` - Name for template variable references
///
/// ## Example
///
/// ```ignore
/// #[directive(
///     selector = "[appHighlight]",
///     standalone = true
/// )]
/// pub struct HighlightDirective {
///     #[input]
///     color: String,
/// }
///
/// impl HighlightDirective {
///     fn on_init(&self) {
///         // Apply highlighting
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn directive(args: TokenStream, input: TokenStream) -> TokenStream {
    directive_macro::directive_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Define a pipe/transform (Angular @Pipe equivalent).
///
/// Pipes transform data in templates.
///
/// ## Attributes
///
/// - `name` (required) - Pipe name used in templates
/// - `pure` - Whether the pipe is pure (default: true)
/// - `standalone` - Whether this is a standalone pipe
///
/// ## Example
///
/// ```ignore
/// #[pipe(name = "uppercase", pure = true)]
/// pub struct UpperCasePipe;
///
/// impl UpperCasePipe {
///     pub fn transform(&self, value: String) -> String {
///         value.to_uppercase()
///     }
/// }
///
/// // Usage in template: {{ name | uppercase }}
/// ```
#[proc_macro_attribute]
pub fn pipe(args: TokenStream, input: TokenStream) -> TokenStream {
    pipe_macro::pipe_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Define a route guard (Angular CanActivate, CanDeactivate, etc.).
///
/// Guards control navigation and route access.
///
/// ## Attributes
///
/// - `type` (required) - Guard type: "can_activate", "can_deactivate", "can_load", "can_activate_child"
/// - `redirect` - URL to redirect to on failure
///
/// ## Example
///
/// ```ignore
/// #[guard(type = "can_activate", redirect = "/login")]
/// pub struct AuthGuard {
///     auth_service: Arc<AuthService>,
/// }
///
/// impl AuthGuard {
///     fn check(&self, route: &ActivatedRouteSnapshot, state: &RouterStateSnapshot) -> GuardResult {
///         if self.auth_service.is_authenticated() {
///             GuardResult::Allow
///         } else {
///             GuardResult::Deny
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn guard(args: TokenStream, input: TokenStream) -> TokenStream {
    guard_macro::guard_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Define an HTTP interceptor (Angular HttpInterceptor equivalent).
///
/// Interceptors modify HTTP requests/responses globally.
///
/// ## Attributes
///
/// - `priority` - Priority in the interceptor chain (higher = earlier)
/// - `paths` - Only intercept specific URL paths
///
/// ## Example
///
/// ```ignore
/// #[interceptor(priority = 100)]
/// pub struct AuthInterceptor {
///     token_service: Arc<TokenService>,
/// }
///
/// impl AuthInterceptor {
///     fn handle(&self, mut request: Request, next: Next) -> ResponseFuture {
///         // Add auth token
///         if let Some(token) = self.token_service.get_token() {
///             request.headers_mut().insert("Authorization", format!("Bearer {}", token));
///         }
///         next(request)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn interceptor(args: TokenStream, input: TokenStream) -> TokenStream {
    interceptor_macro::interceptor_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Define a route resolver (Angular Resolve equivalent).
///
/// Resolvers pre-fetch data before activating a route.
///
/// ## Attributes
///
/// - `resolves` - Description of what this resolver produces
///
/// ## Example
///
/// ```ignore
/// #[resolver(resolves = "User")]
/// pub struct UserResolver {
///     user_service: Arc<UserService>,
/// }
///
/// impl UserResolver {
///     fn load_data(&self, route: &ActivatedRouteSnapshot) -> ResolverResult {
///         let user_id = route.params.get("id").unwrap();
///         match self.user_service.get_user(user_id) {
///             Ok(user) => ResolverResult::Success(Box::new(user)),
///             Err(e) => ResolverResult::Error(e.to_string()),
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn resolver(args: TokenStream, input: TokenStream) -> TokenStream {
    resolver_macro::resolver_impl(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

