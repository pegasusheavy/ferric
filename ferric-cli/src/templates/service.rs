//! Service templates

/// Generate service Rust file
pub fn service_rs(name: &str) -> String {
    format!(
        r#"//! {name} service

use ferric_core::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

/// {name} service for dependency injection
#[derive(Debug, Default)]
pub struct {name} {{
    // Add service state here
}}

impl {name} {{
    /// Create a new {name} service
    pub fn new() -> Self {{
        Self::default()
    }}

    /// Create a shared reference to the service
    pub fn shared() -> Rc<RefCell<Self>> {{
        Rc::new(RefCell::new(Self::new()))
    }}

    // Add service methods here
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_service_creation() {{
        let service = {name}::new();
        // Add tests
    }}
}}
"#,
        name = name
    )
}
