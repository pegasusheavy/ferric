//! Animation states
//!
//! States define the CSS styles for named animation states.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Animation state with associated styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    /// State name ("void", "active", "inactive", "*", etc.)
    pub name: String,
    /// CSS styles for this state
    pub styles: HashMap<String, String>,
}

impl AnimationState {
    /// Create a new animation state
    pub fn new(name: impl Into<String>, styles: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            styles,
        }
    }

    /// Check if this is the wildcard state
    pub fn is_wildcard(&self) -> bool {
        self.name == "*"
    }

    /// Check if this is the void state (element entering/leaving DOM)
    pub fn is_void(&self) -> bool {
        self.name == "void"
    }
}

/// Helper function to create a state
///
/// # Example
///
/// ```rust,no_run
/// use ferric_animations::state;
///
/// let hidden = state("hidden", vec![
///     ("opacity", "0"),
///     ("transform", "translateY(-10px)"),
/// ]);
/// ```
pub fn state(name: impl Into<String>, styles: Vec<(&str, &str)>) -> AnimationState {
    let styles = styles
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    AnimationState::new(name, styles)
}

/// Helper function to create styles
///
/// # Example
///
/// ```rust,no_run
/// use ferric_animations::style;
///
/// let styles = style(vec![
///     ("width", "100px"),
///     ("height", "100px"),
/// ]);
/// ```
pub fn style(styles: Vec<(&str, &str)>) -> HashMap<String, String> {
    styles
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let s = state("active", vec![("opacity", "1")]);
        assert_eq!(s.name, "active");
        assert_eq!(s.styles.get("opacity"), Some(&"1".to_string()));
    }

    #[test]
    fn test_wildcard_state() {
        let s = state("*", vec![]);
        assert!(s.is_wildcard());
        assert!(!s.is_void());
    }

    #[test]
    fn test_void_state() {
        let s = state("void", vec![]);
        assert!(s.is_void());
        assert!(!s.is_wildcard());
    }
}

