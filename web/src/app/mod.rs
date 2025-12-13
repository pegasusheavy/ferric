//! Main application module
//!
//! The app component follows Angular's file organization:
//! - `mod.rs` - Component logic (this file)
//! - `app.component.html` - HTML template
//! - `app.component.scss` - Component styles

mod router;

use ferric_core::reactive::{signal, Signal, effect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Window};

pub use router::{Route, Router};

/// Get the app component HTML template
fn get_template() -> &'static str {
    include_str!("app.component.html")
}

/// The main application
pub struct App {
    document: Document,
    root: Element,
    current_route: Signal<Route>,
    is_mobile_menu_open: Signal<bool>,
    is_dark_mode: Signal<bool>,
}

impl App {
    /// Mount the application to the DOM
    pub fn mount(selector: &str) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");

        let root = document
            .query_selector(selector)?
            .expect("root element not found");

        // Initialize state
        let current_route = signal(Router::parse_route(&window));
        let is_mobile_menu_open = signal(false);
        let is_dark_mode = signal(Self::get_initial_dark_mode(&window));

        let app = App {
            document: document.clone(),
            root,
            current_route,
            is_mobile_menu_open,
            is_dark_mode,
        };

        // Initial render
        app.render()?;

        // Set up routing
        app.setup_routing(&window)?;

        // Set up dark mode
        app.setup_dark_mode(&document)?;

        // Set up event handlers
        app.setup_event_handlers(&document)?;

        // Forget app to keep it alive
        std::mem::forget(app);

        Ok(())
    }

    fn get_initial_dark_mode(window: &Window) -> bool {
        // Check localStorage first
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(value)) = storage.get_item("ferric-dark-mode") {
                return value == "true";
            }
        }
        // Fall back to system preference
        if let Ok(Some(mq)) = window.match_media("(prefers-color-scheme: dark)") {
            return mq.matches();
        }
        false
    }

    fn setup_dark_mode(&self, document: &Document) -> Result<(), JsValue> {
        let html = document.document_element().unwrap();
        let is_dark = self.is_dark_mode.clone();

        effect(move || {
            if is_dark.get() {
                let _ = html.class_list().add_1("dark");
            } else {
                let _ = html.class_list().remove_1("dark");
            }
        });

        Ok(())
    }

    fn setup_routing(&self, window: &Window) -> Result<(), JsValue> {
        let route = self.current_route.clone();
        let window_clone = window.clone();

        // Handle popstate events (browser back/forward)
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            route.set(Router::parse_route(&window_clone));
        }) as Box<dyn Fn(_)>);

        window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }

    fn setup_event_handlers(&self, document: &Document) -> Result<(), JsValue> {
        // Setup navigation link handlers
        self.setup_nav_links(document)?;

        // Setup dark mode toggle
        self.setup_dark_mode_toggle(document)?;

        // Setup mobile menu toggle
        self.setup_mobile_menu_toggle(document)?;

        Ok(())
    }

    fn setup_nav_links(&self, document: &Document) -> Result<(), JsValue> {
        let route = self.current_route.clone();
        let links = document.query_selector_all("[data-link]")?;

        for i in 0..links.length() {
            if let Some(node) = links.get(i) {
                if let Some(element) = node.dyn_ref::<Element>() {
                    let route_clone = route.clone();
                    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                        e.prevent_default();
                        if let Some(target) = e.target() {
                            if let Some(anchor) = target.dyn_ref::<web_sys::HtmlAnchorElement>() {
                                let href = anchor.get_attribute("href").unwrap_or_default();
                                if !href.starts_with("http") {
                                    if let Some(window) = web_sys::window() {
                                        if let Ok(history) = window.history() {
                                            let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&href));
                                        }
                                        route_clone.set(Router::from_path(&href));
                                        window.scroll_to_with_x_and_y(0.0, 0.0);
                                    }
                                }
                            }
                        }
                    }) as Box<dyn Fn(_)>);
                    element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
                    closure.forget();
                }
            }
        }

        Ok(())
    }

    fn setup_dark_mode_toggle(&self, document: &Document) -> Result<(), JsValue> {
        if let Some(toggle) = document.get_element_by_id("dark-mode-toggle") {
            let is_dark = self.is_dark_mode.clone();
            let window = web_sys::window().unwrap();

            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let new_value = !is_dark.get();
                is_dark.set(new_value);
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("ferric-dark-mode", if new_value { "true" } else { "false" });
                }
            }) as Box<dyn Fn(_)>);
            toggle.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }

    fn setup_mobile_menu_toggle(&self, document: &Document) -> Result<(), JsValue> {
        if let Some(toggle) = document.get_element_by_id("mobile-menu-toggle") {
            let menu_open = self.is_mobile_menu_open.clone();

            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                menu_open.set(!menu_open.get());
            }) as Box<dyn Fn(_)>);
            toggle.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }

    fn render(&self) -> Result<(), JsValue> {
        // Render the app template
        self.root.set_inner_html(get_template());

        // Setup event handlers for the rendered content
        self.setup_nav_links(&self.document)?;

        // Render initial page content
        self.render_page_content()?;

        Ok(())
    }

    fn render_page_content(&self) -> Result<(), JsValue> {
        let route = self.current_route.clone();
        let document = self.document.clone();

        effect(move || {
            if let Some(content) = document.get_element_by_id("content") {
                let html = match route.get() {
                    Route::Home => crate::pages::home::render(),
                    Route::Docs => crate::pages::docs::render(),
                    Route::DocsPage(page) => crate::pages::docs::render_page(&page),
                    Route::Guide => crate::pages::guide::render(),
                    Route::Api => crate::pages::api::render(),
                    Route::Examples => crate::pages::examples::render(),
                    Route::NotFound => crate::pages::not_found::render(),
                };
                content.set_inner_html(&html);
            }
        });

        Ok(())
    }
}
