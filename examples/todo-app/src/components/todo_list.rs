//! Todo list component.

use crate::components::{TodoItem, TodoItemEvents};
use crate::models::{Todo, TodoFilter};
use web_sys::Element;
use std::cell::RefCell;
use std::rc::Rc;

/// The main todo list component.
pub struct TodoList {
    element: Element,
    items: Vec<TodoItem>,
}

impl TodoList {
    /// Create a new todo list component wrapping an existing element.
    pub fn new(element: Element) -> Self {
        Self {
            element,
            items: Vec::new(),
        }
    }

    /// Get the DOM element.
    pub fn element(&self) -> &Element {
        &self.element
    }

    /// Render the todo list with the given todos and filter.
    pub fn render(
        &mut self,
        todos: &[Todo],
        filter: TodoFilter,
        on_toggle: impl Fn(u32) + 'static,
        on_delete: impl Fn(u32) + 'static,
        on_edit: impl Fn(u32, String) + 'static,
    ) {
        // Clear existing items
        self.element.set_inner_html("");
        self.items.clear();

        // Create shared event handlers
        let on_toggle = Rc::new(RefCell::new(Box::new(on_toggle) as Box<dyn Fn(u32)>));
        let on_delete = Rc::new(RefCell::new(Box::new(on_delete) as Box<dyn Fn(u32)>));
        let on_edit = Rc::new(RefCell::new(Box::new(on_edit) as Box<dyn Fn(u32, String)>));

        // Filter and render todos
        let filtered_todos: Vec<_> = todos.iter()
            .filter(|t| filter.matches(t))
            .cloned()
            .collect();

        for todo in filtered_todos {
            let events = TodoItemEvents {
                on_toggle: on_toggle.clone(),
                on_delete: on_delete.clone(),
                on_edit: on_edit.clone(),
            };

            let item = TodoItem::new(todo, events);
            self.element.append_child(item.element()).unwrap();
            self.items.push(item);
        }
    }
}
