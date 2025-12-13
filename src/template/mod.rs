//! Template parsing and rendering for Ferric.
//!
//! Provides HTML template parsing with Angular-like syntax for bindings and directives.

mod binding;
mod parser;
mod compiler;

pub use binding::*;
pub use parser::*;
pub use compiler::*;

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

