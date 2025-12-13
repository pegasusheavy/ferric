//! Utility functions for the CLI

/// Convert a string to snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c == '-' || c == ' ' {
            result.push('_');
            prev_is_upper = false;
        } else if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}

/// Convert a string to PascalCase
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '-' || c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a string to kebab-case
pub fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == ' ' {
            result.push('-');
            prev_is_upper = false;
        } else if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyComponent"), "my_component");
        assert_eq!(to_snake_case("my-component"), "my_component");
        assert_eq!(to_snake_case("myComponent"), "my_component");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my-component"), "MyComponent");
        assert_eq!(to_pascal_case("my_component"), "MyComponent");
        assert_eq!(to_pascal_case("myComponent"), "MyComponent");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("MyComponent"), "my-component");
        assert_eq!(to_kebab_case("my_component"), "my-component");
        assert_eq!(to_kebab_case("myComponent"), "my-component");
    }
}
