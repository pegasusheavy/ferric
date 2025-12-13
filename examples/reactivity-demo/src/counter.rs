//! # Counter Demo
//!
//! Demonstrates basic reactive primitives:
//! - `signal()` for mutable state
//! - `computed()` for derived values
//! - `effect()` for side effects
//! - `batch()` for coalescing updates

use ferric_core::reactive::{batch, computed, effect, signal, Computed, Effect, Signal};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, Element};

// Load template and styles at compile time
const TEMPLATE: &str = include_str!("../templates/counter.html");
const STYLES: &str = include_str!("../styles/counter.css");

/// Counter demo showing signals, computed, and effects.
pub struct CounterDemo {
    count: Signal<i32>,
    step: Signal<i32>,
    doubled: Computed<i32>,
    is_even: Computed<bool>,
    display_text: Computed<String>,
    _log_effect: Effect,
    _dom_effect: Effect,
}

impl CounterDemo {
    /// Mount the counter demo to an element.
    pub fn mount(container: &Element) -> Result<(), JsValue> {
        // Inject styles
        inject_styles("counter-styles", STYLES);

        // Create signals
        let count = signal(0);
        let step = signal(1);

        // Create computed values
        let count_for_doubled = count.clone();
        let doubled = computed(move || count_for_doubled.get() * 2);

        let count_for_even = count.clone();
        let is_even = computed(move || count_for_even.get() % 2 == 0);

        let count_for_text = count.clone();
        let doubled_for_text = doubled.clone();
        let is_even_for_text = is_even.clone();
        let display_text = computed(move || {
            let c = count_for_text.get();
            let d = doubled_for_text.get();
            let even = if is_even_for_text.get() { "even" } else { "odd" };
            format!("Count: {} (doubled: {}, {})", c, d, even)
        });

        // Render template
        container.set_inner_html(TEMPLATE);

        // Update initial display
        if let Some(el) = container.query_selector("#counter-display").ok().flatten() {
            el.set_text_content(Some(&display_text.get()));
        }

        // Set up DOM effect
        let display_for_effect = display_text.clone();
        let container_clone = container.clone();
        let _dom_effect = effect(move || {
            let text = display_for_effect.get();
            if let Some(el) = container_clone.query_selector("#counter-display").ok().flatten() {
                el.set_text_content(Some(&text));
            }
        });

        // Set up logging effect
        let count_for_log = count.clone();
        let container_for_log = container.clone();
        let _log_effect = effect(move || {
            let c = count_for_log.get();

            console::log_1(&format!("Count changed to: {}", c).into());

            if let Some(log_el) = container_for_log.query_selector("#effect-log").ok().flatten() {
                let current = log_el.inner_html();
                let timestamp = js_sys::Date::new_0().to_locale_time_string("en-US");
                let new_entry = format!(
                    "<div class='log-entry'>📝 {} - Count: {}</div>",
                    timestamp.as_string().unwrap_or_default(),
                    c
                );
                log_el.set_inner_html(&format!("{}{}", new_entry, current));
            }
        });

        // Set up step display effect
        let step_for_effect = step.clone();
        let container_for_step = container.clone();
        let _step_effect = effect(move || {
            let s = step_for_effect.get();
            if let Some(el) = container_for_step.query_selector("#current-step").ok().flatten() {
                el.set_text_content(Some(&format!("Current step: {}", s)));
            }
        });

        // Set up button handlers
        setup_button_handler(container, "#increment", {
            let count = count.clone();
            let step = step.clone();
            move || {
                let s = step.get();
                count.update(|n| n + s);
            }
        });

        setup_button_handler(container, "#decrement", {
            let count = count.clone();
            let step = step.clone();
            move || {
                let s = step.get();
                count.update(|n| n - s);
            }
        });

        setup_button_handler(container, "#reset", {
            let count = count.clone();
            move || count.set(0)
        });

        for (selector, value) in [("#step-1", 1), ("#step-5", 5), ("#step-10", 10)] {
            setup_button_handler(container, selector, {
                let step = step.clone();
                move || step.set(value)
            });
        }

        setup_button_handler(container, "#batch-update", {
            let count = count.clone();
            move || {
                console::log_1(&"Starting batch update...".into());
                batch(|| {
                    for _ in 0..10 {
                        count.update(|n| n + 1);
                    }
                });
                console::log_1(&"Batch update complete!".into());
            }
        });

        std::mem::forget(CounterDemo {
            count,
            step,
            doubled,
            is_even,
            display_text,
            _log_effect,
            _dom_effect,
        });

        Ok(())
    }
}

fn setup_button_handler<F>(container: &Element, selector: &str, handler: F)
where
    F: Fn() + 'static,
{
    if let Some(button) = container.query_selector(selector).ok().flatten() {
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            handler();
        }) as Box<dyn Fn(_)>);

        let _ = button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

/// Inject CSS styles into the document head.
fn inject_styles(id: &str, css: &str) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        // Check if already injected
        if document.get_element_by_id(id).is_some() {
            return;
        }

        if let Ok(style) = document.create_element("style") {
            let _ = style.set_attribute("id", id);
            style.set_text_content(Some(css));
            if let Some(head) = document.head() {
                let _ = head.append_child(&style);
            }
        }
    }
}
