//! Animation player and execution
//!
//! Manages the execution of animations using the Web Animations API.

use crate::{AnimationState, AnimationTrigger, Transition};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, KeyframeEffect};

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

    /// Apply a transition
    fn apply_transition(
        &self,
        from: &str,
        to: &str,
        transition: &Transition,
    ) -> Result<(), JsValue> {
        // Get from and to states
        let from_state = self.trigger.get_state(from);
        let to_state = self.trigger.get_state(to);

        if from_state.is_none() && to_state.is_none() {
            return Ok(());
        }

        // Create keyframes
        let keyframes = js_sys::Array::new();

        // From keyframe
        if let Some(state) = from_state {
            keyframes.push(&Self::state_to_keyframe(state, 0.0)?);
        }

        // To keyframe
        if let Some(state) = to_state {
            keyframes.push(&Self::state_to_keyframe(state, 1.0)?);
        }

        // Create animation options
        let options = js_sys::Object::new();
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("duration"),
            &JsValue::from_f64(transition.timing.duration as f64),
        )?;
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("delay"),
            &JsValue::from_f64(transition.timing.delay as f64),
        )?;
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("easing"),
            &JsValue::from_str(&transition.timing.easing.to_css()),
        )?;
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("fill"),
            &JsValue::from_str("forwards"),
        )?;

        // Create and play animation
        let animation = self.element.animate_with_keyframes_and_keyframe_animation_options(
            Some(&keyframes),
            &options,
        )?;

        animation.play()?;

        Ok(())
    }

    fn state_to_keyframe(state: &AnimationState, offset: f32) -> Result<JsValue, JsValue> {
        let keyframe = js_sys::Object::new();

        // Set offset
        js_sys::Reflect::set(
            &keyframe,
            &JsValue::from_str("offset"),
            &JsValue::from_f64(offset as f64),
        )?;

        // Set styles
        for (property, value) in &state.styles {
            // Convert CSS property names (e.g., "background-color" to "backgroundColor")
            let camel_case = property
                .split('-')
                .enumerate()
                .map(|(i, part)| {
                    if i == 0 {
                        part.to_string()
                    } else {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    }
                })
                .collect::<String>();

            js_sys::Reflect::set(&keyframe, &JsValue::from_str(&camel_case), &JsValue::from_str(value))?;
        }

        Ok(keyframe.into())
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

