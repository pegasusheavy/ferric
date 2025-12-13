//! Pipe templates

/// Generate pipe Rust file
pub fn pipe_rs(name: &str) -> String {
    format!(
        r#"//! {name} pipe

/// {name} pipe
///
/// Pipes are used to transform data for display.
pub struct {name};

impl {name} {{
    /// Create a new {name} pipe
    pub fn new() -> Self {{
        Self
    }}

    /// Transform the input value
    pub fn transform<T: std::fmt::Display>(&self, value: T) -> String {{
        // Implement your transformation logic here
        value.to_string()
    }}

    /// Transform with additional arguments
    pub fn transform_with_args<T: std::fmt::Display>(
        &self,
        value: T,
        _args: &[&str],
    ) -> String {{
        // Implement your transformation logic with arguments here
        value.to_string()
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

    #[test]
    fn test_transform() {{
        let pipe = {name}::new();
        let result = pipe.transform("test");
        assert_eq!(result, "test");
    }}
}}
"#,
        name = name
    )
}
