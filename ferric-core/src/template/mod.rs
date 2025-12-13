//! Template parsing, compilation, and rendering for Ferric.
//!
//! This module provides a complete template system with Angular-like syntax
//! for bindings and directives.
//!
//! # Features
//!
//! - **Interpolation**: `{{ expression }}` - Display dynamic values
//! - **Property Binding**: `[property]="expression"` - Bind to element properties
//! - **Event Binding**: `(event)="handler()"` - Handle DOM events
//! - **Two-way Binding**: `[(property)]="value"` - Bidirectional data binding
//! - **Attribute Binding**: `[attr.name]="value"` - Bind to HTML attributes
//! - **Class Binding**: `[class.name]="condition"` - Conditionally add CSS classes
//! - **Style Binding**: `[style.property]="value"` - Bind to inline styles
//! - **Template References**: `#ref` - Reference elements in the template
//! - **Structural Directives**: `*ngIf`, `*ngFor` - Control DOM structure
//!
//! # Example
//!
//! ```ignore
//! use ferric::template::{compile_template, TemplateContext};
//!
//! // Create a context with values
//! let context = TemplateContext::new();
//! context.set("name", "World".to_string());
//! context.set("isActive", true);
//! context.set("items", vec!["One", "Two", "Three"]);
//!
//! // Compile and render template
//! let template = r#"
//!     <div class="container" [class.active]="isActive">
//!         <h1>Hello {{ name }}!</h1>
//!         <ul>
//!             <li *ngFor="let item of items">{{ item }}</li>
//!         </ul>
//!         <button (click)="onClick()">Click me</button>
//!     </div>
//! "#;
//!
//! let compiled = compile_template(template).unwrap();
//! let instance = compiled.render(context).unwrap();
//!
//! // Access the root element
//! let root = instance.root();
//!
//! // Update bindings when context changes
//! instance.update();
//! ```

mod binding;
mod compiler;
mod context;
mod expression;
mod instance;
mod parser;

// Re-export binding types
pub use binding::{
    Binding, BindingType, InputBinding, OutputBinding,
    parse_binding_attribute, contains_interpolation, extract_interpolations,
};

// Re-export parser types
pub use parser::{
    parse_template, ParseError,
    TemplateNode, ElementNode, TextNode, InterpolationNode, ComponentNode,
    StructuralDirective,
};

// Re-export compiler types
pub use compiler::{
    compile_template, render, render_static,
    CompiledTemplate, TemplateBuilder,
};

// Re-export context types
pub use context::{
    TemplateContext, ContextBuilder, ContextRef, ContextValue,
};

// Re-export expression evaluator
pub use expression::{
    evaluate, evaluate_to_string, evaluate_to_bool,
    ExprValue, ExpressionEvaluator,
};

// Re-export instance types
pub use instance::{
    TemplateInstance, TemplateRenderer, render_template,
};

/// The delimiter for interpolation expressions (e.g., {{ value }}).
pub const INTERPOLATION_START: &str = "{{";
pub const INTERPOLATION_END: &str = "}}";

/// Prefixes for different binding types.
pub const PROPERTY_BINDING_PREFIX: char = '[';
pub const PROPERTY_BINDING_SUFFIX: char = ']';
pub const EVENT_BINDING_PREFIX: char = '(';
pub const EVENT_BINDING_SUFFIX: char = ')';
pub const TWO_WAY_BINDING_PREFIX: &str = "[(";
pub const TWO_WAY_BINDING_SUFFIX: &str = ")]";
pub const TEMPLATE_REFERENCE_PREFIX: char = '#';
pub const STRUCTURAL_DIRECTIVE_PREFIX: char = '*';

/// Prelude for convenient imports.
pub mod prelude {
    pub use super::{
        // Core types
        compile_template, render, render_static,
        CompiledTemplate, TemplateBuilder,
        TemplateContext, ContextBuilder,
        TemplateInstance,

        // Binding types
        Binding, BindingType,

        // Expression evaluation
        evaluate, evaluate_to_string, evaluate_to_bool,

        // Parser types
        TemplateNode, ElementNode,
    };
}
