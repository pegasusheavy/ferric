//! Todo footer component - works with the footer in the app template.

use crate::models::{TodoFilter, TodoStats};
use crate::template::{on_event, query, query_all};
use web_sys::Element;
use std::cell::RefCell;
use std::rc::Rc;

/// Footer component with filters and item count.
pub struct TodoFooter {
    element: Element,
    on_filter_change: Rc<RefCell<Option<Box<dyn Fn(TodoFilter)>>>>,
    on_clear_completed: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl TodoFooter {
    /// Create a new footer component wrapping an existing element.
    pub fn new(element: Element) -> Self {
        let on_filter_change: Rc<RefCell<Option<Box<dyn Fn(TodoFilter)>>>> =
            Rc::new(RefCell::new(None));
        let on_clear_completed: Rc<RefCell<Option<Box<dyn Fn()>>>> =
            Rc::new(RefCell::new(None));

        // Set up clear button handler
        if let Some(clear_btn) = query(&element, ".clear-completed") {
            let on_clear = on_clear_completed.clone();
            on_event(&clear_btn, "click", move |_: web_sys::Event| {
                if let Some(ref handler) = *on_clear.borrow() {
                    handler();
                }
            });
        }

        // Set up filter handlers
        let filters = query_all(&element, ".filters a");
        for (i, filter_link) in filters.iter().enumerate() {
            let on_filter = on_filter_change.clone();
            let filter = match i {
                0 => TodoFilter::All,
                1 => TodoFilter::Active,
                2 => TodoFilter::Completed,
                _ => TodoFilter::All,
            };

            on_event(filter_link, "click", move |e: web_sys::Event| {
                e.prevent_default();
                if let Some(ref handler) = *on_filter.borrow() {
                    handler(filter);
                }
            });
        }

        Self {
            element,
            on_filter_change,
            on_clear_completed,
        }
    }

    /// Get the DOM element.
    pub fn element(&self) -> &Element {
        &self.element
    }

    /// Set the filter change callback.
    pub fn set_on_filter_change(&self, callback: impl Fn(TodoFilter) + 'static) {
        *self.on_filter_change.borrow_mut() = Some(Box::new(callback));
    }

    /// Set the clear completed callback.
    pub fn set_on_clear_completed(&self, callback: impl Fn() + 'static) {
        *self.on_clear_completed.borrow_mut() = Some(Box::new(callback));
    }

    /// Update the footer with new stats.
    pub fn update(&self, stats: &TodoStats, filter: TodoFilter) {
        // Update count
        if let Some(count_el) = query(&self.element, ".todo-count") {
            let items_text = if stats.active == 1 {
                "1 item left".to_string()
            } else {
                format!("{} items left", stats.active)
            };
            count_el.set_text_content(Some(&items_text));
        }

        // Update filter selection
        let filters = query_all(&self.element, ".filters a");
        for (i, filter_link) in filters.iter().enumerate() {
            let is_selected = match (i, filter) {
                (0, TodoFilter::All) => true,
                (1, TodoFilter::Active) => true,
                (2, TodoFilter::Completed) => true,
                _ => false,
            };

            if is_selected {
                filter_link.set_class_name("selected");
            } else {
                filter_link.set_class_name("");
            }
        }

        // Show/hide clear button
        if let Some(clear_btn) = query(&self.element, ".clear-completed") {
            if stats.completed > 0 {
                clear_btn.set_attribute("style", "").unwrap();
            } else {
                clear_btn.set_attribute("style", "display: none").unwrap();
            }
        }

        // Show/hide footer based on total
        if stats.total > 0 {
            self.element.set_attribute("style", "").unwrap();
        } else {
            self.element.set_attribute("style", "display: none").unwrap();
        }
    }
}
