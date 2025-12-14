//! Ferric Todo App - Benchmark Version
//!
//! A todo application built with Ferric framework for performance comparison.

use ferric_core::reactive::{signal, Signal, computed, Computed, batch};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlInputElement, Window, Performance};

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub created_at: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

#[derive(Clone, Debug, Serialize)]
pub struct Stats {
    pub total: usize,
    pub active: usize,
    pub completed: usize,
}

// ============================================================================
// App State
// ============================================================================

pub struct TodoApp {
    todos: Signal<Vec<Todo>>,
    filter: Signal<Filter>,
    new_todo_text: Signal<String>,
    next_id: Rc<RefCell<u32>>,
    filtered_todos: Computed<Vec<Todo>>,
    stats: Computed<Stats>,
    window: Window,
    document: Document,
    performance: Performance,
}

impl Default for TodoApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoApp {
    pub fn new() -> Self {
        let window = web_sys::window().expect("window");
        let document = window.document().expect("document");
        let performance = window.performance().expect("performance");

        let todos: Signal<Vec<Todo>> = signal(Vec::new());
        let filter = signal(Filter::All);
        let new_todo_text = signal(String::new());

        let todos_clone = todos.clone();
        let filter_clone = filter.clone();

        let filtered_todos = computed(move || {
            let todos = todos_clone.get();
            let filter = filter_clone.get();

            todos.into_iter().filter(|todo| {
                match filter {
                    Filter::All => true,
                    Filter::Active => !todo.completed,
                    Filter::Completed => todo.completed,
                }
            }).collect()
        });

        let todos_clone2 = todos.clone();
        let stats = computed(move || {
            let todos = todos_clone2.get();
            let total = todos.len();
            let completed = todos.iter().filter(|t| t.completed).count();
            Stats {
                total,
                active: total - completed,
                completed,
            }
        });

        Self {
            todos,
            filter,
            new_todo_text,
            next_id: Rc::new(RefCell::new(1)),
            filtered_todos,
            stats,
            window,
            document,
            performance,
        }
    }

    fn mark(&self, name: &str) {
        let _ = js_sys::Reflect::set(
            &js_sys::Reflect::get(&self.window, &"__BENCHMARK__".into()).unwrap_or(JsValue::UNDEFINED),
            &"marks".into(),
            &{
                let marks = js_sys::Reflect::get(
                    &js_sys::Reflect::get(&self.window, &"__BENCHMARK__".into()).unwrap_or(JsValue::UNDEFINED),
                    &"marks".into(),
                ).unwrap_or(JsValue::UNDEFINED);
                let _ = js_sys::Reflect::set(&marks, &name.into(), &self.performance.now().into());
                marks
            },
        );
    }

    pub fn add_todo(&self) {
        let text = self.new_todo_text.get();
        if text.trim().is_empty() {
            return;
        }

        self.mark("add-start");

        let id = {
            let mut next = self.next_id.borrow_mut();
            let id = *next;
            *next += 1;
            id
        };

        let created_at = self.performance.now();
        let text_str = text.trim().to_string();
        self.todos.update(|todos| {
            let mut new_todos = todos.clone();
            new_todos.push(Todo {
                id,
                text: text_str,
                completed: false,
                created_at,
            });
            new_todos
        });

        self.new_todo_text.set(String::new());
        self.render_todos();
        self.render_stats();

        self.mark("add-end");
    }

    pub fn bulk_add(&self, count: u32) {
        self.mark("bulk-add-start");

        batch(|| {
            for i in 0..count {
                let id = {
                    let mut next = self.next_id.borrow_mut();
                    let id = *next;
                    *next += 1;
                    id
                };

                let created_at = self.performance.now();
                let text = format!("Bulk Todo {}", i + 1);
                self.todos.update(|todos| {
                    let mut new_todos = todos.clone();
                    new_todos.push(Todo {
                        id,
                        text,
                        completed: false,
                        created_at,
                    });
                    new_todos
                });
            }
        });

        self.render_todos();
        self.render_stats();

        self.mark("bulk-add-end");
    }

    pub fn toggle_todo(&self, id: u32) {
        self.mark("toggle-start");

        self.todos.update(|todos| {
            todos.iter().map(|t| {
                if t.id == id {
                    Todo { completed: !t.completed, ..t.clone() }
                } else {
                    t.clone()
                }
            }).collect()
        });

        self.render_todos();
        self.render_stats();

        self.mark("toggle-end");
    }

    pub fn toggle_all(&self) {
        self.mark("toggle-all-start");

        let all_completed = self.todos.get().iter().all(|t| t.completed);

        self.todos.update(|todos| {
            todos.iter().map(|t| Todo { completed: !all_completed, ..t.clone() }).collect()
        });

        self.render_todos();
        self.render_stats();

        self.mark("toggle-all-end");
    }

    pub fn delete_todo(&self, id: u32) {
        self.mark("delete-start");

        self.todos.update(|todos| {
            todos.iter().filter(|t| t.id != id).cloned().collect()
        });

        self.render_todos();
        self.render_stats();

        self.mark("delete-end");
    }

    pub fn clear_completed(&self) {
        self.mark("clear-start");

        self.todos.update(|todos| {
            todos.iter().filter(|t| !t.completed).cloned().collect()
        });

        self.render_todos();
        self.render_stats();

        self.mark("clear-end");
    }

    pub fn set_filter(&self, filter: Filter) {
        self.mark("filter-start");
        self.filter.set(filter);
        self.render_todos();
        self.update_filter_buttons();
        self.mark("filter-end");
    }

    fn render_stats(&self) {
        let stats = self.stats.get();

        if let Some(el) = self.document.query_selector(".stats").ok().flatten() {
            el.set_inner_html(&format!(
                "<span>{} total</span><span>{} active</span><span>{} completed</span>",
                stats.total, stats.active, stats.completed
            ));
        }
    }

    fn render_todos(&self) {
        let todos = self.filtered_todos.get();

        if let Some(list) = self.document.query_selector(".todo-list").ok().flatten() {
            let mut html = String::new();

            for todo in &todos {
                let completed_class = if todo.completed { "completed" } else { "" };
                let checked = if todo.completed { "checked" } else { "" };

                html.push_str(&format!(
                    r#"<li class="{}" data-id="{}">
                        <input type="checkbox" {} data-action="toggle" />
                        <span class="todo-text">{}</span>
                        <button class="delete" data-action="delete">×</button>
                    </li>"#,
                    completed_class, todo.id, checked,
                    html_escape(&todo.text)
                ));
            }

            list.set_inner_html(&html);
        }
    }

    fn update_filter_buttons(&self) {
        let filter = self.filter.get();

        if let Ok(buttons) = self.document.query_selector_all(".filters button") {
            for i in 0..buttons.length() {
                if let Some(btn) = buttons.get(i) {
                    let el: Element = btn.dyn_into().unwrap();
                    let is_active = match (i, filter) {
                        (0, Filter::All) => true,
                        (1, Filter::Active) => true,
                        (2, Filter::Completed) => true,
                        _ => false,
                    };

                    if is_active {
                        let _ = el.class_list().add_1("active");
                    } else {
                        let _ = el.class_list().remove_1("active");
                    }
                }
            }
        }
    }

    pub fn mount(&self) {
        self.mark("mount-start");
        self.setup_event_listeners();
        self.render_todos();
        self.render_stats();
        self.update_filter_buttons();
        self.mark("app-mounted");
    }

    fn setup_event_listeners(&self) {
        // This would set up event delegation in a real app
        // For the benchmark, we use global handlers
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ============================================================================
// WASM Entry Point
// ============================================================================

thread_local! {
    static APP: RefCell<Option<Rc<TodoApp>>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("window");

    // Set up benchmark object
    let benchmark = js_sys::Object::new();
    let marks = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&benchmark, &"startTime".into(), &window.performance().unwrap().now().into());
    let _ = js_sys::Reflect::set(&benchmark, &"marks".into(), &marks);
    let _ = js_sys::Reflect::set(&window, &"__BENCHMARK__".into(), &benchmark);

    let app = Rc::new(TodoApp::new());

    // Expose app methods to window for benchmarking
    expose_app_methods(&app);

    APP.with(|a| *a.borrow_mut() = Some(app.clone()));

    app.mount();
}

fn expose_app_methods(app: &Rc<TodoApp>) {
    let window = web_sys::window().expect("window");
    let app_obj = js_sys::Object::new();

    // Bulk add
    let app_clone = app.clone();
    let bulk_add = Closure::wrap(Box::new(move |count: u32| {
        app_clone.bulk_add(count);
    }) as Box<dyn Fn(u32)>);
    let _ = js_sys::Reflect::set(&app_obj, &"bulkAdd".into(), bulk_add.as_ref());
    bulk_add.forget();

    // Toggle all
    let app_clone = app.clone();
    let toggle_all = Closure::wrap(Box::new(move || {
        app_clone.toggle_all();
    }) as Box<dyn Fn()>);
    let _ = js_sys::Reflect::set(&app_obj, &"toggleAll".into(), toggle_all.as_ref());
    toggle_all.forget();

    // Clear completed
    let app_clone = app.clone();
    let clear_completed = Closure::wrap(Box::new(move || {
        app_clone.clear_completed();
    }) as Box<dyn Fn()>);
    let _ = js_sys::Reflect::set(&app_obj, &"clearCompleted".into(), clear_completed.as_ref());
    clear_completed.forget();

    // Set filter
    let app_clone = app.clone();
    let set_filter = Closure::wrap(Box::new(move |filter: String| {
        let f = match filter.as_str() {
            "active" => Filter::Active,
            "completed" => Filter::Completed,
            _ => Filter::All,
        };
        app_clone.set_filter(f);
    }) as Box<dyn Fn(String)>);
    let _ = js_sys::Reflect::set(&app_obj, &"setFilter".into(), set_filter.as_ref());
    set_filter.forget();

    // Get stats
    let app_clone = app.clone();
    let get_stats = Closure::wrap(Box::new(move || -> JsValue {
        let stats = app_clone.stats.get();
        serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL)
    }) as Box<dyn Fn() -> JsValue>);
    let _ = js_sys::Reflect::set(&app_obj, &"getStats".into(), get_stats.as_ref());
    get_stats.forget();

    let _ = js_sys::Reflect::set(&window, &"__APP__".into(), &app_obj);
}

// Event handlers called from JS
#[wasm_bindgen]
pub fn add_todo() {
    APP.with(|app| {
        if let Some(app) = app.borrow().as_ref() {
            // Get text from input
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if let Some(input) = document.query_selector(".new-todo").ok().flatten() {
                let input: HtmlInputElement = input.dyn_into().unwrap();
                app.new_todo_text.set(input.value());
                app.add_todo();
                input.set_value("");
            }
        }
    });
}

#[wasm_bindgen]
pub fn toggle_todo(id: u32) {
    APP.with(|app| {
        if let Some(app) = app.borrow().as_ref() {
            app.toggle_todo(id);
        }
    });
}

#[wasm_bindgen]
pub fn delete_todo(id: u32) {
    APP.with(|app| {
        if let Some(app) = app.borrow().as_ref() {
            app.delete_todo(id);
        }
    });
}

#[wasm_bindgen]
pub fn toggle_all() {
    APP.with(|app| {
        if let Some(app) = app.borrow().as_ref() {
            app.toggle_all();
        }
    });
}

#[wasm_bindgen]
pub fn bulk_add(count: u32) {
    APP.with(|app| {
        if let Some(app) = app.borrow().as_ref() {
            app.bulk_add(count);
        }
    });
}

#[wasm_bindgen]
pub fn clear_completed() {
    APP.with(|app| {
        if let Some(app) = app.borrow().as_ref() {
            app.clear_completed();
        }
    });
}

#[wasm_bindgen]
pub fn set_filter(filter: String) {
    APP.with(|app| {
        if let Some(app) = app.borrow().as_ref() {
            let f = match filter.as_str() {
                "active" => Filter::Active,
                "completed" => Filter::Completed,
                _ => Filter::All,
            };
            app.set_filter(f);
        }
    });
}
