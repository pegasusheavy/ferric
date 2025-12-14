//! Route animations
//!
//! Animations triggered by route changes.

use crate::{AnimationTrigger, TransitionMatcher};
use serde::{Deserialize, Serialize};

/// Route animation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAnimation {
    /// Animation trigger to apply
    pub trigger: AnimationTrigger,
    /// Route path pattern (supports wildcards)
    pub pattern: RoutePattern,
    /// Animation type (enter, leave, or both)
    pub animation_type: RouteAnimationType,
}

impl RouteAnimation {
    /// Create a new route animation
    pub fn new(
        trigger: AnimationTrigger,
        pattern: RoutePattern,
        animation_type: RouteAnimationType,
    ) -> Self {
        Self {
            trigger,
            pattern,
            animation_type,
        }
    }

    /// Check if this animation matches the route transition
    pub fn matches(&self, from: &str, to: &str) -> bool {
        match self.animation_type {
            RouteAnimationType::Enter => self.pattern.matches(to),
            RouteAnimationType::Leave => self.pattern.matches(from),
            RouteAnimationType::Both => self.pattern.matches(from) || self.pattern.matches(to),
        }
    }
}

/// Route pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutePattern {
    /// Exact path match
    Exact(String),
    /// Wildcard match (e.g., "/users/*")
    Wildcard(String),
    /// Any route
    Any,
}

impl RoutePattern {
    /// Check if the pattern matches the route
    pub fn matches(&self, route: &str) -> bool {
        match self {
            Self::Exact(pattern) => route == pattern,
            Self::Wildcard(pattern) => {
                if let Some(prefix) = pattern.strip_suffix('*') {
                    route.starts_with(prefix)
                } else {
                    route == pattern
                }
            }
            Self::Any => true,
        }
    }
}

/// Route animation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteAnimationType {
    /// Animation when entering the route
    Enter,
    /// Animation when leaving the route
    Leave,
    /// Animation for both enter and leave
    Both,
}

/// Helper function to create a route animation
///
/// # Example
///
/// ```rust,no_run
/// use ferric_animations::{route_animation, trigger, state, transition, animate};
///
/// let slide_in = trigger("slideIn", vec![
///     state("void", vec![("transform", "translateX(-100%)")]),
///     state("*", vec![("transform", "translateX(0)")]),
///     transition("void => *", &animate("300ms ease-out")),
/// ]);
///
/// let animation = route_animation(slide_in, "/dashboard/*", RouteAnimationType::Enter);
/// ```
pub fn route_animation(
    trigger: AnimationTrigger,
    pattern: &str,
    animation_type: RouteAnimationType,
) -> RouteAnimation {
    let pattern = if pattern == "*" {
        RoutePattern::Any
    } else if pattern.contains('*') {
        RoutePattern::Wildcard(pattern.to_string())
    } else {
        RoutePattern::Exact(pattern.to_string())
    };

    RouteAnimation::new(trigger, pattern, animation_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_pattern() {
        let pattern = RoutePattern::Exact("/home".to_string());
        assert!(pattern.matches("/home"));
        assert!(!pattern.matches("/about"));
    }

    #[test]
    fn test_wildcard_pattern() {
        let pattern = RoutePattern::Wildcard("/users/*".to_string());
        assert!(pattern.matches("/users/123"));
        assert!(pattern.matches("/users/profile"));
        assert!(!pattern.matches("/admin/users"));
    }

    #[test]
    fn test_any_pattern() {
        let pattern = RoutePattern::Any;
        assert!(pattern.matches("/anything"));
        assert!(pattern.matches("/something/else"));
    }
}

