//! Form bindings for connecting controls to DOM elements.

use crate::controls::{AbstractControl, FormControl};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlTextAreaElement, HtmlSelectElement};

/// Trait for binding form controls to DOM elements.
pub trait FormBinding<T> {
    /// Bind the control to an input element.
    fn bind_to(&self, element: &web_sys::Element) -> Result<(), JsValue>;

    /// Update the DOM element with the current value.
    fn update_view(&self, element: &web_sys::Element);
}

/// Input binding configuration.
pub struct InputBinding {
    /// Update on which event ('input', 'change', 'blur').
    pub update_on: UpdateOn,
    /// Debounce time in milliseconds.
    pub debounce_ms: Option<u32>,
}

/// When to update the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOn {
    /// Update on every input event.
    Input,
    /// Update on change event (blur for text inputs).
    Change,
    /// Update on blur event.
    Blur,
    /// Update on form submit.
    Submit,
}

impl Default for InputBinding {
    fn default() -> Self {
        Self {
            update_on: UpdateOn::Input,
            debounce_ms: None,
        }
    }
}

impl FormBinding<String> for FormControl<String> {
    fn bind_to(&self, element: &web_sys::Element) -> Result<(), JsValue> {
        // Set initial value
        self.update_view(element);

        // Add event listeners
        let control = self.clone();

        // Input event for value changes
        let input_handler = Closure::wrap(Box::new(move |event: Event| {
            if let Some(target) = event.target() {
                let value = if let Some(input) = target.dyn_ref::<HtmlInputElement>() {
                    input.value()
                } else if let Some(textarea) = target.dyn_ref::<HtmlTextAreaElement>() {
                    textarea.value()
                } else if let Some(select) = target.dyn_ref::<HtmlSelectElement>() {
                    select.value()
                } else {
                    return;
                };

                control.set_value(value);
            }
        }) as Box<dyn Fn(Event)>);

        element.add_event_listener_with_callback(
            "input",
            input_handler.as_ref().unchecked_ref(),
        )?;
        input_handler.forget();

        // Blur event for touched state
        let control = self.clone();
        let blur_handler = Closure::wrap(Box::new(move |_event: Event| {
            control.mark_as_touched();
        }) as Box<dyn Fn(Event)>);

        element.add_event_listener_with_callback(
            "blur",
            blur_handler.as_ref().unchecked_ref(),
        )?;
        blur_handler.forget();

        Ok(())
    }

    fn update_view(&self, element: &web_sys::Element) {
        let value = self.value();

        if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
            input.set_value(&value);
        } else if let Some(textarea) = element.dyn_ref::<HtmlTextAreaElement>() {
            textarea.set_value(&value);
        } else if let Some(select) = element.dyn_ref::<HtmlSelectElement>() {
            select.set_value(&value);
        }

        // Update CSS classes
        let class_string = self.state().css_class_string();
        let _ = element.set_attribute("class", &class_string);
    }
}

impl FormBinding<bool> for FormControl<bool> {
    fn bind_to(&self, element: &web_sys::Element) -> Result<(), JsValue> {
        self.update_view(element);

        let control = self.clone();
        let change_handler = Closure::wrap(Box::new(move |event: Event| {
            if let Some(input) = event.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                control.set_value(input.checked());
            }
        }) as Box<dyn Fn(Event)>);

        element.add_event_listener_with_callback(
            "change",
            change_handler.as_ref().unchecked_ref(),
        )?;
        change_handler.forget();

        Ok(())
    }

    fn update_view(&self, element: &web_sys::Element) {
        if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
            input.set_checked(self.value());
        }
    }
}

impl FormBinding<i32> for FormControl<i32> {
    fn bind_to(&self, element: &web_sys::Element) -> Result<(), JsValue> {
        self.update_view(element);

        let control = self.clone();
        let input_handler = Closure::wrap(Box::new(move |event: Event| {
            if let Some(input) = event.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                if let Ok(value) = input.value().parse::<i32>() {
                    control.set_value(value);
                }
            }
        }) as Box<dyn Fn(Event)>);

        element.add_event_listener_with_callback(
            "input",
            input_handler.as_ref().unchecked_ref(),
        )?;
        input_handler.forget();

        Ok(())
    }

    fn update_view(&self, element: &web_sys::Element) {
        if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
            input.set_value(&self.value().to_string());
        }
    }
}

impl FormBinding<f64> for FormControl<f64> {
    fn bind_to(&self, element: &web_sys::Element) -> Result<(), JsValue> {
        self.update_view(element);

        let control = self.clone();
        let input_handler = Closure::wrap(Box::new(move |event: Event| {
            if let Some(input) = event.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                if let Ok(value) = input.value().parse::<f64>() {
                    control.set_value(value);
                }
            }
        }) as Box<dyn Fn(Event)>);

        element.add_event_listener_with_callback(
            "input",
            input_handler.as_ref().unchecked_ref(),
        )?;
        input_handler.forget();

        Ok(())
    }

    fn update_view(&self, element: &web_sys::Element) {
        if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
            input.set_value(&self.value().to_string());
        }
    }
}

