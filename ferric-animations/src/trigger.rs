//! Animation triggers
//!
//! Triggers are named collections of states and transitions.

use crate::{AnimationState, Transition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Animation trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrigger {
    /// Trigger name
    pub name: String,
    /// States in this trigger
    pub states: HashMap<String, AnimationState>,
    /// Transitions between states
    pub transitions: Vec<Transition>,
}

impl AnimationTrigger {
    /// Create a new animation trigger
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            states: HashMap::new(),
            transitions: Vec::new(),
        }
    }

    /// Add a state to the trigger
    pub fn add_state(&mut self, state: AnimationState) {
        self.states.insert(state.name.clone(), state);
    }

    /// Add a transition to the trigger
    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    /// Get a state by name
    pub fn get_state(&self, name: &str) -> Option<&AnimationState> {
        self.states.get(name)
    }

    /// Find transitions that match the state change
    pub fn find_transitions(&self, from: &str, to: &str) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| t.matches(from, to))
            .collect()
    }
}

/// Metadata for all animation definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationMetadata {
    /// All triggers
    pub triggers: HashMap<String, AnimationTrigger>,
}

impl AnimationMetadata {
    /// Create new animation metadata
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
        }
    }

    /// Add a trigger
    pub fn add_trigger(&mut self, trigger: AnimationTrigger) {
        self.triggers.insert(trigger.name.clone(), trigger);
    }

    /// Get a trigger by name
    pub fn get_trigger(&self, name: &str) -> Option<&AnimationTrigger> {
        self.triggers.get(name)
    }
}

impl Default for AnimationMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a trigger
///
/// # Example
///
/// ```rust,no_run
/// use ferric_animations::{trigger, state, transition, animate};
///
/// let fade = trigger("fade", vec![
///     state("void", vec![("opacity", "0")]),
///     state("*", vec![("opacity", "1")]),
///     transition("void => *", &animate("300ms ease-in")),
/// ]);
/// ```
pub fn trigger(
    name: impl Into<String>,
    definitions: Vec<AnimationDefinition>,
) -> AnimationTrigger {
    let mut trigger = AnimationTrigger::new(name);

    for def in definitions {
        match def {
            AnimationDefinition::State(state) => trigger.add_state(state),
            AnimationDefinition::Transition(transition) => trigger.add_transition(transition),
        }
    }

    trigger
}

/// Animation definition (state or transition)
pub enum AnimationDefinition {
    State(AnimationState),
    Transition(Transition),
}

// Allow states to be converted to definitions
impl From<AnimationState> for AnimationDefinition {
    fn from(state: AnimationState) -> Self {
        AnimationDefinition::State(state)
    }
}

// Allow transitions to be converted to definitions
impl From<Transition> for AnimationDefinition {
    fn from(transition: Transition) -> Self {
        AnimationDefinition::Transition(transition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state, transition};

    #[test]
    fn test_trigger_creation() {
        let mut trigger = AnimationTrigger::new("fade");
        trigger.add_state(state("hidden", vec![("opacity", "0")]));
        trigger.add_state(state("visible", vec![("opacity", "1")]));

        assert_eq!(trigger.states.len(), 2);
        assert!(trigger.get_state("hidden").is_some());
    }

    #[test]
    fn test_find_transitions() {
        let mut trigger = AnimationTrigger::new("slide");
        trigger.add_transition(transition("left => right", "300ms"));
        trigger.add_transition(transition("* => left", "200ms"));

        let matches = trigger.find_transitions("center", "left");
        assert_eq!(matches.len(), 1);
    }
}

