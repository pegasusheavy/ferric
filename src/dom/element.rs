//! Element creation and manipulation utilities.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Create an element with the given tag name.
pub fn create_element(tag: &str) -> web_sys::Element {
    super::document()
        .create_element(tag)
        .expect("should create element")
}

/// Create a text node.
pub fn create_text(content: &str) -> web_sys::Text {
    super::document().create_text_node(content)
}

/// Create a document fragment.
pub fn create_fragment() -> web_sys::DocumentFragment {
    super::document().create_document_fragment()
}

/// Set an attribute on an element.
pub fn set_attribute(element: &web_sys::Element, name: &str, value: &str) -> Result<(), JsValue> {
    element.set_attribute(name, value)
}

/// Remove an attribute from an element.
pub fn remove_attribute(element: &web_sys::Element, name: &str) -> Result<(), JsValue> {
    element.remove_attribute(name)
}

/// Get an attribute from an element.
pub fn get_attribute(element: &web_sys::Element, name: &str) -> Option<String> {
    element.get_attribute(name)
}

/// Check if an element has an attribute.
pub fn has_attribute(element: &web_sys::Element, name: &str) -> bool {
    element.has_attribute(name)
}

/// Set the inner HTML of an element.
pub fn set_inner_html(element: &web_sys::Element, html: &str) {
    element.set_inner_html(html);
}

/// Get the inner HTML of an element.
pub fn get_inner_html(element: &web_sys::Element) -> String {
    element.inner_html()
}

/// Set the text content of an element.
pub fn set_text_content(element: &web_sys::Element, text: &str) {
    element.set_text_content(Some(text));
}

/// Get the text content of an element.
pub fn get_text_content(element: &web_sys::Element) -> Option<String> {
    element.text_content()
}

/// Add a class to an element.
pub fn add_class(element: &web_sys::Element, class: &str) -> Result<(), JsValue> {
    element.class_list().add_1(class)
}

/// Remove a class from an element.
pub fn remove_class(element: &web_sys::Element, class: &str) -> Result<(), JsValue> {
    element.class_list().remove_1(class)
}

/// Toggle a class on an element.
pub fn toggle_class(element: &web_sys::Element, class: &str) -> Result<bool, JsValue> {
    element.class_list().toggle(class)
}

/// Check if an element has a class.
pub fn has_class(element: &web_sys::Element, class: &str) -> bool {
    element.class_list().contains(class)
}

/// Set a style property on an element.
pub fn set_style(element: &web_sys::Element, property: &str, value: &str) -> Result<(), JsValue> {
    if let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>() {
        html_element.style().set_property(property, value)
    } else {
        Ok(())
    }
}

/// Get a style property from an element.
pub fn get_style(element: &web_sys::Element, property: &str) -> String {
    if let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>() {
        html_element
            .style()
            .get_property_value(property)
            .unwrap_or_default()
    } else {
        String::new()
    }
}

/// Append a child to an element.
pub fn append_child(parent: &web_sys::Element, child: &web_sys::Node) -> Result<(), JsValue> {
    parent.append_child(child)?;
    Ok(())
}

/// Remove a child from an element.
pub fn remove_child(parent: &web_sys::Element, child: &web_sys::Node) -> Result<(), JsValue> {
    parent.remove_child(child)?;
    Ok(())
}

/// Insert a node before another node.
pub fn insert_before(
    parent: &web_sys::Element,
    new_node: &web_sys::Node,
    reference_node: Option<&web_sys::Node>,
) -> Result<(), JsValue> {
    parent.insert_before(new_node, reference_node)?;
    Ok(())
}

/// Replace a child node.
pub fn replace_child(
    parent: &web_sys::Element,
    new_node: &web_sys::Node,
    old_node: &web_sys::Node,
) -> Result<(), JsValue> {
    parent.replace_child(new_node, old_node)?;
    Ok(())
}

/// Remove an element from the DOM.
pub fn remove(element: &web_sys::Element) {
    element.remove();
}

