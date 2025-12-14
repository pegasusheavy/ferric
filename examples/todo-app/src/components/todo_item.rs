//! Individual todo item component with inline template.

use crate::models::Todo;
use crate::template::{html, on_event, query};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement, KeyboardEvent};
use std::cell::RefCell;
use std::rc::Rc;

/// Events emitted by a todo item.
pub struct TodoItemEvents {
    pub on_toggle: Rc<RefCell<Box<dyn Fn(u32)>>>,
    pub on_delete: Rc<RefCell<Box<dyn Fn(u32)>>>,
    pub on_edit: Rc<RefCell<Box<dyn Fn(u32, String)>>>,
}

/// HTML template for a todo item.
fn template(todo: &Todo) -> String {
    let completed_class = if todo.completed { " completed" } else { "" };
    let checked = if todo.completed { "checked" } else { "" };

    format!(r#"
        <li class="todo-item{completed_class}" data-id="{id}">
            <div class="view">
                <input class="toggle" type="checkbox" {checked} />
                <label class="todo-label">{text}</label>
                <button class="destroy">×</button>
            </div>
            <input class="edit" type="text" value="{text}" />
        </li>
    "#,
        id = todo.id,
        text = html_escape(&todo.text),
        completed_class = completed_class,
        checked = checked,
    )
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// A single todo item component.
pub struct TodoItem {
    element: Element,
    todo: Todo,
    editing: Rc<RefCell<bool>>,
}

impl TodoItem {
    /// Create a new todo item component.
    pub fn new(todo: Todo, events: TodoItemEvents) -> Self {
        let element = html(&template(&todo));
        let editing = Rc::new(RefCell::new(false));

        // Get child elements
        let checkbox = query(&element, ".toggle");
        let label = query(&element, ".todo-label");
        let destroy = query(&element, ".destroy");
        let edit_input = query(&element, ".edit");

        let todo_id = todo.id;
        let completed = todo.completed;

        // Toggle handler
        if let Some(cb) = checkbox {
            let on_toggle = events.on_toggle.clone();
            on_event(&cb, "click", move |_: web_sys::Event| {
                (on_toggle.borrow())(todo_id);
            });
        }

        // Double-click to edit
        if let Some(lbl) = label {
            let element_clone = element.clone();
            let editing_clone = editing.clone();
            let edit_input_clone = edit_input.clone();

            on_event(&lbl, "dblclick", move |_: web_sys::Event| {
                *editing_clone.borrow_mut() = true;
                element_clone.set_class_name("todo-item editing");

                if let Some(ref input) = edit_input_clone
                    && let Some(input) = input.dyn_ref::<HtmlInputElement>() {
                        let _ = input.focus();
                        input.select();
                    }
            });
        }

        // Delete handler
        if let Some(del) = destroy {
            let on_delete = events.on_delete.clone();
            on_event(&del, "click", move |_: web_sys::Event| {
                (on_delete.borrow())(todo_id);
            });
        }

        // Edit handlers
        if let Some(ref input) = edit_input {
            // Keydown handler
            let element_clone = element.clone();
            let editing_clone = editing.clone();
            let on_edit = events.on_edit.clone();
            let input_clone = input.clone();

            on_event(input, "keydown", move |e: web_sys::Event| {
                let event: KeyboardEvent = e.unchecked_into();
                let input_el = input_clone.dyn_ref::<HtmlInputElement>().unwrap();

                if event.key() == "Enter" {
                    let value = input_el.value().trim().to_string();
                    if !value.is_empty() {
                        (on_edit.borrow())(todo_id, value);
                    }
                    *editing_clone.borrow_mut() = false;
                    let class = if completed { "todo-item completed" } else { "todo-item" };
                    element_clone.set_class_name(class);
                } else if event.key() == "Escape" {
                    *editing_clone.borrow_mut() = false;
                    let class = if completed { "todo-item completed" } else { "todo-item" };
                    element_clone.set_class_name(class);
                }
            });

            // Blur handler
            let element_clone = element.clone();
            let editing_clone = editing.clone();
            let on_edit = events.on_edit.clone();
            let input_clone = input.clone();

            on_event(input, "blur", move |_: web_sys::Event| {
                if *editing_clone.borrow() {
                    let input_el = input_clone.dyn_ref::<HtmlInputElement>().unwrap();
                    let value = input_el.value().trim().to_string();
                    if !value.is_empty() {
                        (on_edit.borrow())(todo_id, value);
                    }
                    *editing_clone.borrow_mut() = false;
                    let class = if completed { "todo-item completed" } else { "todo-item" };
                    element_clone.set_class_name(class);
                }
            });
        }

        Self { element, todo, editing }
    }

    /// Get the DOM element.
    pub fn element(&self) -> &Element {
        &self.element
    }

    /// Get the todo data.
    pub fn todo(&self) -> &Todo {
        &self.todo
    }
}
