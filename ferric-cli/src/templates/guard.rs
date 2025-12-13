//! Guard templates

/// Generate guard Rust file
pub fn guard_rs(name: &str) -> String {
    format!(
        r#"//! {name} guard

use std::future::Future;
use std::pin::Pin;

/// Result type for guard checks
pub type GuardResult = Pin<Box<dyn Future<Output = bool>>>;

/// {name} guard
///
/// Guards are used to control access to routes or features.
pub struct {name};

impl {name} {{
    /// Create a new {name} guard
    pub fn new() -> Self {{
        Self
    }}

    /// Check if access should be allowed
    pub fn can_activate(&self) -> GuardResult {{
        Box::pin(async {{
            // Implement your guard logic here
            // Return true to allow access, false to deny

            true
        }})
    }}

    /// Check if the user can leave the current route
    pub fn can_deactivate(&self) -> GuardResult {{
        Box::pin(async {{
            // Implement your guard logic here
            // Return true to allow navigation away, false to prevent

            true
        }})
    }}
}}

impl Default for {name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_can_activate() {{
        let guard = {name}::new();
        let result = guard.can_activate().await;
        assert!(result);
    }}
}}
"#,
        name = name
    )
}
