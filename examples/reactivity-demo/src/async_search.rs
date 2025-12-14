//! # Async Search Demo
//!
//! Demonstrates async data fetching:
//! - Debounced input
//! - Loading and error states
//! - Automatic refetching when dependencies change

use ferric_core::reactive::{computed, effect, signal, Computed, Effect, Signal};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::Element;
use std::cell::RefCell;
use std::rc::Rc;

// Load template and styles at compile time
const TEMPLATE: &str = include_str!("../templates/async_search.html");
const STYLES: &str = include_str!("../styles/async_search.css");

const MOCK_DATA: &[&str] = &[
    "Rust Programming Language",
    "WebAssembly (WASM)",
    "Ferric Framework",
    "Angular",
    "React",
    "Vue.js",
    "Svelte",
    "SolidJS",
    "TypeScript",
    "JavaScript",
    "Node.js",
    "Deno",
    "Cargo",
    "npm",
    "pnpm",
    "Webpack",
    "Vite",
    "esbuild",
    "SWC",
];

pub struct AsyncSearchDemo {
    query: Signal<String>,
    results: Signal<Vec<String>>,
    is_loading: Signal<bool>,
    error: Signal<Option<String>>,
    debounced_query: Signal<String>,
    result_count: Computed<usize>,
    _effects: Vec<Effect>,
}

impl AsyncSearchDemo {
    async fn search(query: String) -> Result<Vec<String>, String> {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let window = web_sys::window().unwrap();
            let closure = Closure::once(Box::new(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnOnce()>);
            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                500,
            ).unwrap();
            closure.forget();
        });

        wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|_| "Timeout error".to_string())?;

        let query_lower = query.to_lowercase();
        let results: Vec<String> = if query_lower.is_empty() {
            vec![]
        } else {
            MOCK_DATA
                .iter()
                .filter(|item| item.to_lowercase().contains(&query_lower))
                .map(|s| s.to_string())
                .collect()
        };

        Ok(results)
    }

    pub fn mount(container: &Element) -> Result<(), JsValue> {
        inject_styles("async-search-styles", STYLES);

        let query = signal(String::new());
        let results: Signal<Vec<String>> = signal(vec![]);
        let is_loading = signal(false);
        let error: Signal<Option<String>> = signal(None);
        let debounced_query = signal(String::new());

        let results_for_count = results.clone();
        let result_count = computed(move || results_for_count.get().len());

        // Render template
        container.set_inner_html(TEMPLATE);

        // Set up input handler
        if let Some(input) = container.query_selector("#search-input").ok().flatten() {
            let query = query.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                if let Some(target) = e.target()
                    && let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                        query.set(input.value());
                    }
            }) as Box<dyn Fn(_)>);

            let _ = input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        // Debounce effect
        let query_for_debounce = query.clone();
        let debounced_query_setter = debounced_query.clone();
        let container_for_debounce = container.clone();
        let timeout_id: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
        let timeout_id_clone = Rc::clone(&timeout_id);

        let debounce_effect = effect(move || {
            let q = query_for_debounce.get();

            if let Some(id) = timeout_id_clone.borrow_mut().take()
                && let Some(window) = web_sys::window() {
                    window.clear_timeout_with_handle(id);
                }

            if let Some(el) = container_for_debounce.query_selector("#debounce-indicator").ok().flatten() {
                if !q.is_empty() {
                    el.set_text_content(Some("⏱️ Debouncing..."));
                } else {
                    el.set_text_content(None);
                }
            }

            let debounced = debounced_query_setter.clone();
            let container = container_for_debounce.clone();
            let timeout_ref = Rc::clone(&timeout_id_clone);

            let closure = Closure::once(Box::new(move || {
                debounced.set(q);
                *timeout_ref.borrow_mut() = None;

                if let Some(el) = container.query_selector("#debounce-indicator").ok().flatten() {
                    el.set_text_content(None);
                }
            }) as Box<dyn FnOnce()>);

            if let Some(window) = web_sys::window()
                && let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    300,
                ) {
                    *timeout_id_clone.borrow_mut() = Some(id);
                }

            closure.forget();
        });

        // Search effect
        let debounced_for_search = debounced_query.clone();
        let results_for_search = results.clone();
        let is_loading_for_search = is_loading.clone();
        let error_for_search = error.clone();

        let search_effect = effect(move || {
            let q = debounced_for_search.get();

            if q.is_empty() {
                results_for_search.set(vec![]);
                return;
            }

            is_loading_for_search.set(true);
            error_for_search.set(None);

            let results = results_for_search.clone();
            let is_loading = is_loading_for_search.clone();
            let error = error_for_search.clone();

            spawn_local(async move {
                match Self::search(q).await {
                    Ok(r) => {
                        results.set(r);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        results.set(vec![]);
                    }
                }
                is_loading.set(false);
            });
        });

        // Loading indicator effect
        let is_loading_for_ui = is_loading.clone();
        let container_for_loading = container.clone();
        let loading_effect = effect(move || {
            let loading = is_loading_for_ui.get();
            if let Some(el) = container_for_loading.query_selector("#loading-indicator").ok().flatten() {
                if loading {
                    let _ = el.class_list().remove_1("hidden");
                } else {
                    let _ = el.class_list().add_1("hidden");
                }
            }
        });

        // Results effect
        let results_for_ui = results.clone();
        let container_for_results = container.clone();
        let results_effect = effect(move || {
            let r = results_for_ui.get();
            if let Some(el) = container_for_results.query_selector("#search-results").ok().flatten() {
                if r.is_empty() {
                    el.set_inner_html(r#"<p class="placeholder-text">No results found</p>"#);
                } else {
                    let html: String = r.iter()
                        .map(|item| format!(r#"<div class="result-item">{}</div>"#, item))
                        .collect();
                    el.set_inner_html(&html);
                }
            }
        });

        // Result count effect
        let result_count_for_ui = result_count.clone();
        let debounced_for_count = debounced_query.clone();
        let container_for_count = container.clone();
        let count_effect = effect(move || {
            let count = result_count_for_ui.get();
            let q = debounced_for_count.get();

            if let Some(el) = container_for_count.query_selector("#result-count").ok().flatten() {
                if q.is_empty() {
                    el.set_text_content(None);
                } else {
                    el.set_text_content(Some(&format!(
                        "Found {} result{}",
                        count,
                        if count == 1 { "" } else { "s" }
                    )));
                }
            }
        });

        // Error effect
        let error_for_ui = error.clone();
        let container_for_error = container.clone();
        let error_effect = effect(move || {
            let err = error_for_ui.get();
            if let Some(el) = container_for_error.query_selector("#search-error").ok().flatten() {
                if let Some(e) = err {
                    el.set_text_content(Some(&format!("Error: {}", e)));
                    let _ = el.class_list().remove_1("hidden");
                } else {
                    let _ = el.class_list().add_1("hidden");
                }
            }
        });

        std::mem::forget(AsyncSearchDemo {
            query,
            results,
            is_loading,
            error,
            debounced_query,
            result_count,
            _effects: vec![
                debounce_effect,
                search_effect,
                loading_effect,
                results_effect,
                count_effect,
                error_effect,
            ],
        });

        Ok(())
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
