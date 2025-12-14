//! Main application logic using reactive store and templated rendering.

use crate::models::TodoFilter;
use crate::store::TodoStore;
use crate::template::{document, html, inject_styles, on_event, query, set_text};
use ferric_core::reactive::{effect, Effect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement, KeyboardEvent};

/// CSS styles for the todo application.
const STYLES: &str = include_str!("components/todo_app.css");

/// Main application template using Angular-like syntax comments.
/// The reactive bindings are handled programmatically for now.
const TEMPLATE: &str = r##"
<section class="todoapp">
    <header class="header">
        <h1>todos</h1>
        <input
            class="new-todo"
            placeholder="What needs to be done?"
            autofocus
        >
    </header>

    <section class="main">
        <input id="toggle-all" class="toggle-all" type="checkbox">
        <label for="toggle-all">Mark all as complete</label>
        <ul class="todo-list">
            <!-- Todo items rendered reactively -->
        </ul>
    </section>

    <footer class="footer">
        <span class="todo-count">
            <!-- e.g., "2 items left" -->
        </span>
        <ul class="filters">
            <li><a href="#/" class="filter-all">All</a></li>
            <li><a href="#/active" class="filter-active">Active</a></li>
            <li><a href="#/completed" class="filter-completed">Completed</a></li>
        </ul>
        <button class="clear-completed">Clear completed</button>
    </footer>
</section>
"##;

/// Todo item template.
fn todo_item_template(id: u32, text: &str, completed: bool, editing: bool) -> String {
    let completed_class = if completed { "completed" } else { "" };
    let editing_class = if editing { "editing" } else { "" };
    let checked = if completed { "checked" } else { "" };

    format!(
        r#"<li class="{} {}" data-id="{}">
            <div class="view">
                <input class="toggle" type="checkbox" {}>
                <label>{}</label>
                <button class="destroy"></button>
            </div>
            <input class="edit" value="{}">
        </li>"#,
        completed_class, editing_class, id, checked, text, text
    )
}

/// The main todo application with reactive state.
pub struct TodoApp {
    /// The reactive store.
    store: Rc<TodoStore>,
    /// Root element.
    root: Option<Element>,
    /// Effects that need to stay alive.
    _effects: Vec<Effect>,
}

impl TodoApp {
    /// Create a new todo application.
    pub fn new() -> Self {
        Self {
            store: Rc::new(TodoStore::new()),
            root: None,
            _effects: Vec::new(),
        }
    }

    /// Mount the application to a DOM element.
    pub fn mount(mut self, selector: &str) -> Result<(), JsValue> {
        let document = document();
        let root = document
            .query_selector(selector)?
            .ok_or("Element not found")?;

        // Inject styles
        inject_styles("todo-app", STYLES);

        // Render initial template
        let app_element = html(TEMPLATE);
        root.set_inner_html("");
        root.append_child(&app_element)?;
        self.root = Some(app_element.clone());

        // Set up reactive effects
        self.setup_input_binding(&app_element);
        self.setup_toggle_all_binding(&app_element);
        self.setup_filter_bindings(&app_element);
        self.setup_clear_completed(&app_element);

        // Set up reactive rendering effects
        self.setup_list_effect(&app_element);
        self.setup_footer_effect(&app_element);
        self.setup_visibility_effects(&app_element);

        // Keep the app alive
        std::mem::forget(self);

        Ok(())
    }

    /// Set up the new todo input binding.
    fn setup_input_binding(&mut self, app: &Element) {
        if let Some(input) = query(app, ".new-todo") {
            let store = Rc::clone(&self.store);

            // Handle input changes
            let store_for_input = Rc::clone(&store);
            on_event(&input, "input", move |e: web_sys::Event| {
                if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                    store_for_input.set_new_todo_text(input.value());
                }
            });

            // Handle Enter key
            let store_for_enter = Rc::clone(&store);
            on_event(&input, "keydown", move |e: web_sys::Event| {
                let event: KeyboardEvent = e.unchecked_into();
                if event.key() == "Enter" {
                    store_for_enter.add_todo();
                }
            });

            // Effect to clear input when new_todo_text is cleared
            let input_clone = input.clone();
            let store_for_effect = Rc::clone(&store);
            let fx = effect(move || {
                let text = store_for_effect.new_todo_text.get();
                if let Ok(input) = input_clone.clone().dyn_into::<HtmlInputElement>()
                    && input.value() != text {
                        input.set_value(&text);
                    }
            });
            self._effects.push(fx);
        }
    }

    /// Set up the toggle-all checkbox binding.
    fn setup_toggle_all_binding(&mut self, app: &Element) {
        if let Some(toggle) = query(app, ".toggle-all") {
            let store = Rc::clone(&self.store);

            // Handle click
            let store_for_click = Rc::clone(&store);
            on_event(&toggle, "change", move |_: web_sys::Event| {
                store_for_click.toggle_all();
            });

            // Effect to sync checkbox state
            let toggle_clone = toggle.clone();
            let store_for_effect = Rc::clone(&store);
            let fx = effect(move || {
                let all_completed = store_for_effect.all_completed.get();
                if let Ok(checkbox) = toggle_clone.clone().dyn_into::<HtmlInputElement>() {
                    checkbox.set_checked(all_completed);
                }
            });
            self._effects.push(fx);
        }
    }

    /// Set up filter link bindings.
    fn setup_filter_bindings(&mut self, app: &Element) {
        let filters = [
            (".filter-all", TodoFilter::All),
            (".filter-active", TodoFilter::Active),
            (".filter-completed", TodoFilter::Completed),
        ];

        for (selector, filter) in filters {
            if let Some(link) = query(app, selector) {
                let store = Rc::clone(&self.store);
                let filter_value = filter;

                on_event(&link, "click", move |e: web_sys::Event| {
                    e.prevent_default();
                    store.set_filter(filter_value);
                });
            }
        }

        // Effect to update selected filter class
        let app_clone = app.clone();
        let store = Rc::clone(&self.store);
        let fx = effect(move || {
            let current_filter = store.filter.get();

            for (selector, filter) in &[
                (".filter-all", TodoFilter::All),
                (".filter-active", TodoFilter::Active),
                (".filter-completed", TodoFilter::Completed),
            ] {
                if let Some(link) = query(&app_clone, selector) {
                    let class_list = link.class_list();
                    if *filter == current_filter {
                        let _ = class_list.add_1("selected");
                    } else {
                        let _ = class_list.remove_1("selected");
                    }
                }
            }
        });
        self._effects.push(fx);
    }

    /// Set up clear completed button.
    fn setup_clear_completed(&mut self, app: &Element) {
        if let Some(button) = query(app, ".clear-completed") {
            let store = Rc::clone(&self.store);

            on_event(&button, "click", move |_: web_sys::Event| {
                store.clear_completed();
            });

            // Effect to show/hide based on completed count
            let button_clone = button.clone();
            let store_for_effect = Rc::clone(&self.store);
            let fx = effect(move || {
                let has_completed = store_for_effect.has_completed.get();
                let display = if has_completed { "block" } else { "none" };
                let _ = button_clone.set_attribute("style", &format!("display: {}", display));
            });
            self._effects.push(fx);
        }
    }

    /// Set up the reactive todo list rendering.
    fn setup_list_effect(&mut self, app: &Element) {
        if let Some(list) = query(app, ".todo-list") {
            let store = Rc::clone(&self.store);
            let list_clone = list.clone();

            let fx = effect(move || {
                let filtered = store.filtered_todos.get();
                let editing_id = store.editing_id.get();

                // Clear and rebuild list
                list_clone.set_inner_html("");

                for todo in &filtered {
                    let is_editing = editing_id == Some(todo.id);
                    let item_html = todo_item_template(todo.id, &todo.text, todo.completed, is_editing);

                    let temp = document().create_element("div").unwrap();
                    temp.set_inner_html(&item_html);

                    if let Some(item) = temp.first_element_child() {
                        // Set up item event handlers
                        setup_todo_item_handlers(&item, todo.id, &store);

                        // Focus edit input if editing
                        if is_editing
                            && let Some(edit_input) = query(&item, ".edit")
                                && let Ok(input) = edit_input.dyn_into::<HtmlInputElement>() {
                                    input.set_value(&store.edit_text.get());
                                    let _ = input.focus();
                                    let len = input.value().len() as u32;
                                    let _ = input.set_selection_range(len, len);
                                }

                        let _ = list_clone.append_child(&item);
                    }
                }
            });
            self._effects.push(fx);
        }
    }

    /// Set up the reactive footer rendering.
    fn setup_footer_effect(&mut self, app: &Element) {
        if let Some(count_el) = query(app, ".todo-count") {
            let store = Rc::clone(&self.store);

            let fx = effect(move || {
                let text = store.items_left_text.get();
                set_text(&count_el, &text);
            });
            self._effects.push(fx);
        }
    }

    /// Set up visibility effects for main and footer.
    fn setup_visibility_effects(&mut self, app: &Element) {
        // Main section visibility
        if let Some(main) = query(app, ".main") {
            let store = Rc::clone(&self.store);
            let fx = effect(move || {
                let show = store.show_main.get();
                let display = if show { "block" } else { "none" };
                let _ = main.set_attribute("style", &format!("display: {}", display));
            });
            self._effects.push(fx);
        }

        // Footer visibility
        if let Some(footer) = query(app, ".footer") {
            let store = Rc::clone(&self.store);
            let fx = effect(move || {
                let show = store.show_footer.get();
                let display = if show { "block" } else { "none" };
                let _ = footer.set_attribute("style", &format!("display: {}", display));
            });
            self._effects.push(fx);
        }
    }
}

/// Set up event handlers for a todo item.
fn setup_todo_item_handlers(item: &Element, id: u32, store: &Rc<TodoStore>) {
    // Toggle checkbox
    if let Some(toggle) = query(item, ".toggle") {
        let store = Rc::clone(store);
        on_event(&toggle, "change", move |_: web_sys::Event| {
            store.toggle_todo(id);
        });
    }

    // Delete button
    if let Some(destroy) = query(item, ".destroy") {
        let store = Rc::clone(store);
        on_event(&destroy, "click", move |_: web_sys::Event| {
            store.delete_todo(id);
        });
    }

    // Double-click to edit
    if let Some(label) = query(item, "label") {
        let store = Rc::clone(store);
        on_event(&label, "dblclick", move |_: web_sys::Event| {
            store.start_editing(id);
        });
    }

    // Edit input
    if let Some(edit_input) = query(item, ".edit") {
        // Handle input changes
        let store_for_input = Rc::clone(store);
        on_event(&edit_input, "input", move |e: web_sys::Event| {
            if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                store_for_input.set_edit_text(input.value());
            }
        });

        // Handle blur (commit edit)
        let store_for_blur = Rc::clone(store);
        on_event(&edit_input, "blur", move |_: web_sys::Event| {
            store_for_blur.commit_edit();
        });

        // Handle keyboard
        let store_for_key = Rc::clone(store);
        on_event(&edit_input, "keydown", move |e: web_sys::Event| {
            let event: KeyboardEvent = e.unchecked_into();
            match event.key().as_str() {
                "Enter" => store_for_key.commit_edit(),
                "Escape" => store_for_key.cancel_edit(),
                _ => {}
            }
        });
    }
}

impl Default for TodoApp {
    fn default() -> Self {
        Self::new()
    }
}
