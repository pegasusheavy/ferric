//! # Timer Demo
//!
//! Demonstrates:
//! - Effect cleanup (stopping intervals)
//! - `untracked()` to read without creating dependencies
//! - Multiple independent timers

use ferric_core::reactive::{effect, signal, untracked, Effect, Signal};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, Element};
use std::cell::RefCell;
use std::rc::Rc;

// Load template and styles at compile time
const TEMPLATE: &str = include_str!("../templates/timer.html");
const STYLES: &str = include_str!("../styles/timer.css");

pub struct TimerDemo {
    counter: Signal<i32>,
    is_running: Signal<bool>,
    stopwatch_ms: Signal<u32>,
    stopwatch_running: Signal<bool>,
    lap_times: Signal<Vec<u32>>,
    countdown: Signal<i32>,
    countdown_running: Signal<bool>,
    _effects: Vec<Effect>,
}

impl TimerDemo {
    pub fn mount(container: &Element) -> Result<(), JsValue> {
        inject_styles("timer-styles", STYLES);

        let counter = signal(0);
        let is_running = signal(false);
        let interval_id: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

        let stopwatch_ms = signal(0u32);
        let stopwatch_running = signal(false);
        let lap_times: Signal<Vec<u32>> = signal(vec![]);
        let stopwatch_interval_id: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

        let countdown = signal(10);
        let countdown_running = signal(false);
        let countdown_interval_id: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

        // Render template
        container.set_inner_html(TEMPLATE);

        // === Counter Timer ===
        let counter_for_effect = counter.clone();
        let container_for_counter = container.clone();
        let counter_display_effect = effect(move || {
            let c = counter_for_effect.get();
            if let Some(el) = container_for_counter.query_selector("#counter-display").ok().flatten() {
                el.set_text_content(Some(&c.to_string()));
            }
        });

        let is_running_for_effect = is_running.clone();
        let counter_for_interval = counter.clone();
        let interval_id_clone = Rc::clone(&interval_id);

        let counter_interval_effect = effect(move || {
            let running = is_running_for_effect.get();
            let interval_ref = Rc::clone(&interval_id_clone);

            if let Some(id) = interval_ref.borrow_mut().take()
                && let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(id);
                    console::log_1(&"Counter interval cleared".into());
                }

            if running {
                let counter = counter_for_interval.clone();

                let closure = Closure::wrap(Box::new(move || {
                    let current = untracked(|| counter.get());
                    counter.set(current + 1);
                }) as Box<dyn Fn()>);

                if let Some(window) = web_sys::window()
                    && let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        1000,
                    ) {
                        *interval_ref.borrow_mut() = Some(id);
                        console::log_1(&"Counter interval started".into());
                    }

                closure.forget();
            }
        });

        setup_toggle_button(container, "#counter-toggle", is_running.clone());
        setup_reset_button(container, "#counter-reset", counter.clone(), is_running.clone(), 0);

        let is_running_for_btn = is_running.clone();
        let container_for_btn = container.clone();
        let toggle_btn_effect = effect(move || {
            let running = is_running_for_btn.get();
            if let Some(btn) = container_for_btn.query_selector("#counter-toggle").ok().flatten() {
                btn.set_text_content(Some(if running { "Stop" } else { "Start" }));
            }
        });

        // === Stopwatch ===
        let stopwatch_for_effect = stopwatch_ms.clone();
        let container_for_stopwatch = container.clone();
        let stopwatch_display_effect = effect(move || {
            let ms = stopwatch_for_effect.get();
            let minutes = ms / 60000;
            let seconds = (ms % 60000) / 1000;
            let centis = (ms % 1000) / 10;

            if let Some(el) = container_for_stopwatch.query_selector("#stopwatch-display").ok().flatten() {
                el.set_text_content(Some(&format!("{:02}:{:02}.{:02}", minutes, seconds, centis)));
            }
        });

        let stopwatch_running_for_effect = stopwatch_running.clone();
        let stopwatch_ms_for_interval = stopwatch_ms.clone();
        let stopwatch_interval_clone = Rc::clone(&stopwatch_interval_id);

        let stopwatch_interval_effect = effect(move || {
            let running = stopwatch_running_for_effect.get();
            let interval_ref = Rc::clone(&stopwatch_interval_clone);

            if let Some(id) = interval_ref.borrow_mut().take()
                && let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(id);
                }

            if running {
                let ms = stopwatch_ms_for_interval.clone();

                let closure = Closure::wrap(Box::new(move || {
                    let current = untracked(|| ms.get());
                    ms.set(current + 10);
                }) as Box<dyn Fn()>);

                if let Some(window) = web_sys::window()
                    && let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        10,
                    ) {
                        *interval_ref.borrow_mut() = Some(id);
                    }

                closure.forget();
            }
        });

        let lap_times_for_effect = lap_times.clone();
        let container_for_laps = container.clone();
        let laps_display_effect = effect(move || {
            let laps = lap_times_for_effect.get();
            if let Some(el) = container_for_laps.query_selector("#lap-times").ok().flatten() {
                if laps.is_empty() {
                    el.set_inner_html("");
                } else {
                    let html: String = laps.iter().enumerate().rev().map(|(i, ms)| {
                        let minutes = ms / 60000;
                        let seconds = (ms % 60000) / 1000;
                        let centis = (ms % 1000) / 10;
                        format!(
                            r#"<div class="lap-item">Lap {} - {:02}:{:02}.{:02}</div>"#,
                            i + 1, minutes, seconds, centis
                        )
                    }).collect();
                    el.set_inner_html(&html);
                }
            }
        });

        setup_toggle_button(container, "#stopwatch-toggle", stopwatch_running.clone());

        if let Some(btn) = container.query_selector("#stopwatch-lap").ok().flatten() {
            let lap_times = lap_times.clone();
            let stopwatch_ms = stopwatch_ms.clone();
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let current_time = stopwatch_ms.get();
                lap_times.mutate(|laps| laps.push(current_time));
            }) as Box<dyn Fn(_)>);
            let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        if let Some(btn) = container.query_selector("#stopwatch-reset").ok().flatten() {
            let stopwatch_ms = stopwatch_ms.clone();
            let stopwatch_running = stopwatch_running.clone();
            let lap_times = lap_times.clone();
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                ferric_core::reactive::batch(|| {
                    stopwatch_running.set(false);
                    stopwatch_ms.set(0);
                    lap_times.set(vec![]);
                });
            }) as Box<dyn Fn(_)>);
            let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        let stopwatch_running_for_btn = stopwatch_running.clone();
        let container_for_sw_btn = container.clone();
        let sw_toggle_btn_effect = effect(move || {
            let running = stopwatch_running_for_btn.get();
            if let Some(btn) = container_for_sw_btn.query_selector("#stopwatch-toggle").ok().flatten() {
                btn.set_text_content(Some(if running { "Stop" } else { "Start" }));
            }
        });

        // === Countdown ===
        let countdown_for_effect = countdown.clone();
        let container_for_countdown = container.clone();
        let countdown_display_effect = effect(move || {
            let c = countdown_for_effect.get();
            if let Some(el) = container_for_countdown.query_selector("#countdown-display").ok().flatten() {
                el.set_text_content(Some(&c.to_string()));

                let class_list = el.class_list();
                if c <= 3 && c > 0 {
                    let _ = class_list.add_1("warning");
                } else {
                    let _ = class_list.remove_1("warning");
                }
            }
        });

        let countdown_running_for_effect = countdown_running.clone();
        let countdown_for_interval = countdown.clone();
        let countdown_interval_clone = Rc::clone(&countdown_interval_id);
        let countdown_running_setter = countdown_running.clone();

        let countdown_interval_effect = effect(move || {
            let running = countdown_running_for_effect.get();
            let interval_ref = Rc::clone(&countdown_interval_clone);

            if let Some(id) = interval_ref.borrow_mut().take()
                && let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(id);
                }

            if running {
                let cd = countdown_for_interval.clone();
                let cd_running = countdown_running_setter.clone();

                let closure = Closure::wrap(Box::new(move || {
                    let current = untracked(|| cd.get());
                    if current > 0 {
                        cd.set(current - 1);
                    } else {
                        cd_running.set(false);
                        console::log_1(&"⏰ Countdown finished!".into());
                    }
                }) as Box<dyn Fn()>);

                if let Some(window) = web_sys::window()
                    && let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        1000,
                    ) {
                        *interval_ref.borrow_mut() = Some(id);
                    }

                closure.forget();
            }
        });

        setup_toggle_button(container, "#countdown-toggle", countdown_running.clone());
        setup_reset_button(container, "#countdown-reset", countdown.clone(), countdown_running.clone(), 10);

        for time in [5, 10, 30, 60] {
            let selector = format!(".preset[data-time='{}']", time);
            if let Some(btn) = container.query_selector(&selector).ok().flatten() {
                let countdown = countdown.clone();
                let countdown_running = countdown_running.clone();
                let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    ferric_core::reactive::batch(|| {
                        countdown_running.set(false);
                        countdown.set(time);
                    });
                }) as Box<dyn Fn(_)>);
                let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }

        let countdown_running_for_btn = countdown_running.clone();
        let container_for_cd_btn = container.clone();
        let cd_toggle_btn_effect = effect(move || {
            let running = countdown_running_for_btn.get();
            if let Some(btn) = container_for_cd_btn.query_selector("#countdown-toggle").ok().flatten() {
                btn.set_text_content(Some(if running { "Stop" } else { "Start" }));
            }
        });

        std::mem::forget(TimerDemo {
            counter,
            is_running,
            stopwatch_ms,
            stopwatch_running,
            lap_times,
            countdown,
            countdown_running,
            _effects: vec![
                counter_display_effect,
                counter_interval_effect,
                toggle_btn_effect,
                stopwatch_display_effect,
                stopwatch_interval_effect,
                laps_display_effect,
                sw_toggle_btn_effect,
                countdown_display_effect,
                countdown_interval_effect,
                cd_toggle_btn_effect,
            ],
        });

        Ok(())
    }
}

fn setup_toggle_button(container: &Element, selector: &str, is_running: Signal<bool>) {
    if let Some(btn) = container.query_selector(selector).ok().flatten() {
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            is_running.update(|r| !r);
        }) as Box<dyn Fn(_)>);
        let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

fn setup_reset_button(
    container: &Element,
    selector: &str,
    value: Signal<i32>,
    is_running: Signal<bool>,
    reset_value: i32,
) {
    if let Some(btn) = container.query_selector(selector).ok().flatten() {
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            ferric_core::reactive::batch(|| {
                is_running.set(false);
                value.set(reset_value);
            });
        }) as Box<dyn Fn(_)>);
        let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

fn inject_styles(id: &str, css: &str) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
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
