//! DOM querying utilities.

use wasm_bindgen::JsCast;

/// Query for a single element by selector.
pub fn query_selector(selector: &str) -> Option<web_sys::Element> {
    super::document().query_selector(selector).ok().flatten()
}

/// Query for a single element within a parent.
pub fn query_selector_from(parent: &web_sys::Element, selector: &str) -> Option<web_sys::Element> {
    parent.query_selector(selector).ok().flatten()
}

/// Query for all elements matching a selector.
pub fn query_selector_all(selector: &str) -> Vec<web_sys::Element> {
    let node_list = match super::document().query_selector_all(selector) {
        Ok(list) => list,
        Err(_) => return Vec::new(),
    };

    let mut elements = Vec::new();
    for i in 0..node_list.length() {
        if let Some(element) = node_list.item(i)
            && let Ok(el) = element.dyn_into::<web_sys::Element>() {
                elements.push(el);
            }
    }
    elements
}

/// Query for all elements matching a selector within a parent.
pub fn query_selector_all_from(parent: &web_sys::Element, selector: &str) -> Vec<web_sys::Element> {
    let node_list = match parent.query_selector_all(selector) {
        Ok(list) => list,
        Err(_) => return Vec::new(),
    };

    let mut elements = Vec::new();
    for i in 0..node_list.length() {
        if let Some(element) = node_list.item(i)
            && let Ok(el) = element.dyn_into::<web_sys::Element>() {
                elements.push(el);
            }
    }
    elements
}

/// Get an element by its ID.
pub fn get_element_by_id(id: &str) -> Option<web_sys::Element> {
    super::document().get_element_by_id(id)
}

/// Get elements by class name.
pub fn get_elements_by_class_name(class: &str) -> Vec<web_sys::Element> {
    let collection = super::document().get_elements_by_class_name(class);

    let mut elements = Vec::new();
    for i in 0..collection.length() {
        if let Some(element) = collection.item(i) {
            elements.push(element);
        }
    }
    elements
}

/// Get elements by tag name.
pub fn get_elements_by_tag_name(tag: &str) -> Vec<web_sys::Element> {
    let collection = super::document().get_elements_by_tag_name(tag);

    let mut elements = Vec::new();
    for i in 0..collection.length() {
        if let Some(element) = collection.item(i) {
            elements.push(element);
        }
    }
    elements
}

/// Find the closest ancestor matching a selector.
pub fn closest(element: &web_sys::Element, selector: &str) -> Option<web_sys::Element> {
    element.closest(selector).ok().flatten()
}

/// Check if an element matches a selector.
pub fn matches(element: &web_sys::Element, selector: &str) -> bool {
    element.matches(selector).unwrap_or(false)
}

