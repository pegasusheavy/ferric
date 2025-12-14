//! Animation player and execution
//!
//! Manages the execution of animations using CSS transitions.

use crate::{AnimationState, AnimationTrigger, Transition};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;

/// Animation player
pub struct AnimationPlayer {
    element: Element,
    trigger: Rc<AnimationTrigger>,
    current_state: Option<String>,
}

impl AnimationPlayer {
    /// Create a new animation player
    pub fn new(element: Element, trigger: Rc<AnimationTrigger>) -> Self {
        Self {
            element,
            trigger,
            current_state: None,
        }
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, to_state: &str) -> Result<(), JsValue> {
        let from_state = self.current_state.as_deref().unwrap_or("void");

        // Find matching transitions
        let transitions = self.trigger.find_transitions(from_state, to_state);

        if transitions.is_empty() {
            // No transition found, apply state directly
            self.apply_state(to_state)?;
        } else {
            // Apply first matching transition
            let transition = transitions[0];
            self.apply_transition(from_state, to_state, transition)?;
        }

        self.current_state = Some(to_state.to_string());
        Ok(())
    }

    /// Apply a state directly
    fn apply_state(&self, state_name: &str) -> Result<(), JsValue> {
        if let Some(state) = self.trigger.get_state(state_name) {
            for (property, value) in &state.styles {
                self.element
                    .dyn_ref::<web_sys::HtmlElement>()
                    .ok_or_else(|| JsValue::from_str("Element is not an HtmlElement"))?
                    .style()
                    .set_property(property, value)?;
            }
        }
        Ok(())
    }

    /// Apply a transition using CSS
    fn apply_transition(
        &self,
        from: &str,
        to: &str,
        transition: &Transition,
    ) -> Result<(), JsValue> {
        let html_element = self
            .element
            .dyn_ref::<web_sys::HtmlElement>()
            .ok_or_else(|| JsValue::from_str("Element is not an HtmlElement"))?;

        let style = html_element.style();

        // Apply from state first
        if let Some(state) = self.trigger.get_state(from) {
            for (property, value) in &state.styles {
                style.set_property(property, value)?;
            }
        }

        // Set up CSS transition
        let transition_value = format!(
            "all {}ms {} {}ms",
            transition.timing.duration,
            transition.timing.easing.to_css(),
            transition.timing.delay
        );
        style.set_property("transition", &transition_value)?;

        // Apply to state
        if let Some(state) = self.trigger.get_state(to) {
            for (property, value) in &state.styles {
                style.set_property(property, value)?;
            }
        }

        Ok(())
    }

    /// Get the current state
    pub fn current_state(&self) -> Option<&str> {
        self.current_state.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_animation_player_creation() {
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");
        let element = document.create_element("div").expect("create element");

        let trigger = Rc::new(AnimationTrigger::new("test"));
        let player = AnimationPlayer::new(element, trigger);

        assert!(player.current_state().is_none());
    }
}

