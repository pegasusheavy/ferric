//! Template rendering utilities for creating DOM from HTML strings.

use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlStyleElement};

/// Render an HTML string to a DOM element.
pub fn html(template: &str) -> Element {
    let document = document();
    let container = document.create_element("div").unwrap();
    container.set_inner_html(template.trim());

    // Return the first child element, or the container if no children
    container
        .first_element_child()
        .unwrap_or(container)
}

/// Render an HTML string and return the container with all children.
pub fn html_fragment(template: &str) -> Element {
    let document = document();
    let container = document.create_element("div").unwrap();
    container.set_inner_html(template.trim());
    container
}

/// Inject scoped CSS styles into the document.
pub fn inject_styles(component_name: &str, css: &str) {
    let document = document();
    let style_id = format!("style-{}", component_name);

    // Check if styles already injected
    if document.get_element_by_id(&style_id).is_some() {
        return;
    }

    let style = document
        .create_element("style")
        .unwrap()
        .dyn_into::<HtmlStyleElement>()
        .unwrap();

    style.set_id(&style_id);
    style.set_inner_html(css);

    if let Some(head) = document.head() {
        let _ = head.append_child(&style);
    }
}

/// Query selector helper.
pub fn query(element: &Element, selector: &str) -> Option<Element> {
    element.query_selector(selector).ok().flatten()
}

/// Query all matching elements.
pub fn query_all(element: &Element, selector: &str) -> Vec<Element> {
    let node_list = element.query_selector_all(selector).ok();
    let mut elements = Vec::new();

    if let Some(list) = node_list {
        for i in 0..list.length() {
            if let Some(node) = list.get(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    elements.push(el);
                }
            }
        }
    }

    elements
}

/// Get the document.
pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

/// Create an element with classes.
pub fn create_element(tag: &str, class: &str) -> Element {
    let el = document().create_element(tag).unwrap();
    if !class.is_empty() {
        el.set_class_name(class);
    }
    el
}

/// Set element text content.
pub fn set_text(element: &Element, text: &str) {
    element.set_text_content(Some(text));
}

/// Set element attribute.
pub fn set_attr(element: &Element, name: &str, value: &str) {
    element.set_attribute(name, value).unwrap();
}

/// Add event listener helper.
pub fn on_event<F>(element: &Element, event: &str, handler: F)
where
    F: FnMut(web_sys::Event) + 'static,
{
    use wasm_bindgen::prelude::*;

    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    element
        .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

/// Trait for components with templates.
pub trait Component {
    /// The component's HTML template.
    fn template(&self) -> String;

    /// The component's CSS styles (optional).
    fn styles(&self) -> &'static str {
        ""
    }

    /// Render the component to a DOM element.
    fn render(&self) -> Element {
        let styles = self.styles();
        if !styles.is_empty() {
            inject_styles(std::any::type_name::<Self>(), styles);
        }
        html(&self.template())
    }

    /// Called after the component is mounted to the DOM.
    fn on_mount(&mut self, _element: &Element) {}
}

