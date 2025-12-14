//! Animation builder utilities
//!
//! Fluent API for building complex animations.

use crate::{
    AnimationState, AnimationTrigger, KeyframeSequence, StaggerConfig, Transition,
    TransitionMatcher,
};

/// Builder for creating animation triggers
pub struct AnimationBuilder {
    trigger: AnimationTrigger,
}

impl AnimationBuilder {
    /// Create a new animation builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            trigger: AnimationTrigger::new(name),
        }
    }

    /// Add a state
    pub fn state(mut self, name: impl Into<String>, styles: Vec<(&str, &str)>) -> Self {
        let styles = styles
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.trigger.add_state(AnimationState::new(name, styles));
        self
    }

    /// Add a transition
    pub fn transition(mut self, pattern: &str, timing: &str) -> Self {
        let matcher = TransitionMatcher::parse(pattern).expect("Invalid transition pattern");
        let timing = crate::timing::TimingFunction::parse(timing).expect("Invalid timing");
        self.trigger
            .add_transition(Transition::new(matcher, timing, None));
        self
    }

    /// Add a transition with animation name
    pub fn transition_with_animation(
        mut self,
        pattern: &str,
        timing: &str,
        animation: impl Into<String>,
    ) -> Self {
        let matcher = TransitionMatcher::parse(pattern).expect("Invalid transition pattern");
        let timing = crate::timing::TimingFunction::parse(timing).expect("Invalid timing");
        self.trigger
            .add_transition(Transition::new(matcher, timing, Some(animation.into())));
        self
    }

    /// Build the trigger
    pub fn build(self) -> AnimationTrigger {
        self.trigger
    }
}

/// Builder for keyframe sequences
pub struct KeyframeBuilder {
    sequence: KeyframeSequence,
}

impl KeyframeBuilder {
    /// Create a new keyframe builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            sequence: KeyframeSequence::new(name),
        }
    }

    /// Add a keyframe at the given offset
    pub fn at(mut self, offset: f32, styles: Vec<(&str, &str)>) -> Self {
        let styles = styles
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.sequence
            .add_keyframe(crate::keyframes::Keyframe::new(offset, styles));
        self
    }

    /// Build the keyframe sequence
    pub fn build(self) -> KeyframeSequence {
        self.sequence
    }
}

/// Builder for stagger configurations
pub struct StaggerBuilder {
    timing: crate::timing::TimingFunction,
    delay: u32,
    max_items: Option<usize>,
    direction: crate::stagger::StaggerDirection,
}

impl StaggerBuilder {
    /// Create a new stagger builder
    pub fn new(duration: u32, stagger_delay: u32) -> Self {
        Self {
            timing: crate::timing::TimingFunction::new(
                duration,
                0,
                crate::timing::EasingFunction::Ease,
            ),
            delay: stagger_delay,
            max_items: None,
            direction: crate::stagger::StaggerDirection::Forward,
        }
    }

    /// Set the easing function
    pub fn easing(mut self, easing: crate::timing::EasingFunction) -> Self {
        self.timing.easing = easing;
        self
    }

    /// Set the base delay
    pub fn base_delay(mut self, delay: u32) -> Self {
        self.timing.delay = delay;
        self
    }

    /// Set maximum items
    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    /// Set direction to reverse
    pub fn reverse(mut self) -> Self {
        self.direction = crate::stagger::StaggerDirection::Reverse;
        self
    }

    /// Build the stagger configuration
    pub fn build(self) -> StaggerConfig {
        let mut config = StaggerConfig::new(self.timing, self.delay);
        if let Some(max) = self.max_items {
            config = config.with_max_items(max);
        }
        config.with_direction(self.direction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_builder() {
        let trigger = AnimationBuilder::new("fade")
            .state("hidden", vec![("opacity", "0")])
            .state("visible", vec![("opacity", "1")])
            .transition("hidden => visible", "300ms ease-in")
            .build();

        assert_eq!(trigger.name, "fade");
        assert_eq!(trigger.states.len(), 2);
        assert_eq!(trigger.transitions.len(), 1);
    }

    #[test]
    fn test_keyframe_builder() {
        let sequence = KeyframeBuilder::new("bounce")
            .at(0.0, vec![("transform", "translateY(0)")])
            .at(0.5, vec![("transform", "translateY(-20px)")])
            .at(1.0, vec![("transform", "translateY(0)")])
            .build();

        assert_eq!(sequence.name, "bounce");
        assert_eq!(sequence.keyframes.len(), 3);
    }

    #[test]
    fn test_stagger_builder() {
        let config = StaggerBuilder::new(300, 50)
            .max_items(10)
            .reverse()
            .build();

        assert_eq!(config.stagger_delay, 50);
        assert_eq!(config.max_items, Some(10));
        assert_eq!(
            config.direction,
            crate::stagger::StaggerDirection::Reverse
        );
    }
}

