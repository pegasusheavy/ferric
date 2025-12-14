//! Animation transitions
//!
//! Transitions define how to animate between states.

use crate::timing::TimingFunction;
use serde::{Deserialize, Serialize};

/// Transition between animation states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// State transition pattern (e.g., "* => active", "void => *")
    pub matcher: TransitionMatcher,
    /// Animation timing
    pub timing: TimingFunction,
    /// Optional animation name for keyframes
    pub animation: Option<String>,
}

impl Transition {
    /// Create a new transition
    pub fn new(
        matcher: TransitionMatcher,
        timing: TimingFunction,
        animation: Option<String>,
    ) -> Self {
        Self {
            matcher,
            timing,
            animation,
        }
    }

    /// Check if this transition matches the state change
    pub fn matches(&self, from: &str, to: &str) -> bool {
        self.matcher.matches(from, to)
    }
}

/// Matcher for state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionMatcher {
    /// Bidirectional: "state1 <=> state2"
    Bidirectional(String, String),
    /// Unidirectional: "state1 => state2"
    Unidirectional(String, String),
    /// Any transition: "*"
    Any,
    /// From any state: "* => state"
    FromAny(String),
    /// To any state: "state => *"
    ToAny(String),
    /// Increment: ":increment"
    Increment,
    /// Decrement: ":decrement"
    Decrement,
}

impl TransitionMatcher {
    /// Parse a transition pattern
    pub fn parse(pattern: &str) -> Result<Self, String> {
        let pattern = pattern.trim();

        // Special cases
        if pattern == "*" {
            return Ok(Self::Any);
        }
        if pattern == ":increment" {
            return Ok(Self::Increment);
        }
        if pattern == ":decrement" {
            return Ok(Self::Decrement);
        }

        // Bidirectional
        if let Some((from, to)) = pattern.split_once("<=>") {
            return Ok(Self::Bidirectional(
                from.trim().to_string(),
                to.trim().to_string(),
            ));
        }

        // Unidirectional
        if let Some((from, to)) = pattern.split_once("=>") {
            let from = from.trim().to_string();
            let to = to.trim().to_string();

            return Ok(if from == "*" && to != "*" {
                Self::FromAny(to)
            } else if from != "*" && to == "*" {
                Self::ToAny(from)
            } else {
                Self::Unidirectional(from, to)
            });
        }

        Err(format!("Invalid transition pattern: {}", pattern))
    }

    /// Check if this matcher matches the state transition
    pub fn matches(&self, from: &str, to: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Bidirectional(s1, s2) => {
                (from == s1 && to == s2) || (from == s2 && to == s1)
            }
            Self::Unidirectional(f, t) => from == f && to == t,
            Self::FromAny(t) => to == t,
            Self::ToAny(f) => from == f,
            Self::Increment => {
                // Numeric increment
                if let (Ok(from_num), Ok(to_num)) = (from.parse::<i32>(), to.parse::<i32>()) {
                    to_num > from_num
                } else {
                    false
                }
            }
            Self::Decrement => {
                // Numeric decrement
                if let (Ok(from_num), Ok(to_num)) = (from.parse::<i32>(), to.parse::<i32>()) {
                    to_num < from_num
                } else {
                    false
                }
            }
        }
    }
}

/// Helper function to create a transition
///
/// # Example
///
/// ```rust,no_run
/// use ferric_animations::{transition, animate};
///
/// let t = transition("void => *", animate("300ms ease-in"));
/// ```
pub fn transition(pattern: &str, timing: &str) -> Transition {
    let matcher = TransitionMatcher::parse(pattern).expect("Invalid transition pattern");
    let timing = TimingFunction::parse(timing).expect("Invalid timing");
    Transition::new(matcher, timing, None)
}

/// Helper function to create animation timing
pub fn animate(timing: &str) -> String {
    timing.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unidirectional() {
        let m = TransitionMatcher::parse("active => inactive").unwrap();
        assert!(m.matches("active", "inactive"));
        assert!(!m.matches("inactive", "active"));
    }

    #[test]
    fn test_parse_bidirectional() {
        let m = TransitionMatcher::parse("active <=> inactive").unwrap();
        assert!(m.matches("active", "inactive"));
        assert!(m.matches("inactive", "active"));
    }

    #[test]
    fn test_parse_from_any() {
        let m = TransitionMatcher::parse("* => active").unwrap();
        assert!(m.matches("anything", "active"));
        assert!(!m.matches("active", "anything"));
    }

    #[test]
    fn test_increment_decrement() {
        let inc = TransitionMatcher::Increment;
        assert!(inc.matches("1", "2"));
        assert!(!inc.matches("2", "1"));

        let dec = TransitionMatcher::Decrement;
        assert!(dec.matches("2", "1"));
        assert!(!dec.matches("1", "2"));
    }
}

