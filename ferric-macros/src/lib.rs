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
//! ## Example
//!
//! ```ignore
//! use ferric_core::prelude::*;
//!
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
//! ```

mod component;
mod component_file;
mod injectable;
mod input_output;
mod host;
mod view_query;
mod utils;

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

