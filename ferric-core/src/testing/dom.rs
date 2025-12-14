//! DOM testing utilities.
//!
//! Provides helpers for interacting with and asserting on DOM elements in tests.

use std::collections::HashMap;
use std::fmt;

/// Simulated DOM for testing.
pub struct TestDom {
    root: DomNode,
    next_id: usize,
}

impl TestDom {
    /// Create a new test DOM.
    pub fn new() -> Self {
        Self {
            root: DomNode::element("body"),
            next_id: 1,
        }
    }

    /// Get the root element.
    pub fn root(&self) -> &DomNode {
        &self.root
    }

    /// Get a mutable reference to the root element.
    pub fn root_mut(&mut self) -> &mut DomNode {
        &mut self.root
    }

    /// Create an element node.
    pub fn create_element(&mut self, tag: &str) -> DomNode {
        let id = self.next_id;
        self.next_id += 1;
        DomNode {
            id,
            node_type: NodeType::Element {
                tag: tag.to_string(),
                attributes: HashMap::new(),
                children: Vec::new(),
            },
        }
    }

    /// Create a text node.
    pub fn create_text(&mut self, text: &str) -> DomNode {
        let id = self.next_id;
        self.next_id += 1;
        DomNode {
            id,
            node_type: NodeType::Text(text.to_string()),
        }
    }

    /// Query for elements by selector.
    pub fn query(&self, selector: &str) -> Vec<&DomNode> {
        self.root.query_selector(selector)
    }

    /// Query for a single element.
    pub fn query_one(&self, selector: &str) -> Option<&DomNode> {
        self.query(selector).into_iter().next()
    }

    /// Get HTML representation.
    pub fn to_html(&self) -> String {
        self.root.to_html()
    }

    /// Set inner HTML on root.
    pub fn set_html(&mut self, html: &str) {
        // Parse HTML and set as children
        // This is a simplified implementation
        self.root = parse_html(html);
    }
}

impl Default for TestDom {
    fn default() -> Self {
        Self::new()
    }
}

/// A node in the test DOM.
#[derive(Clone)]
pub struct DomNode {
    pub id: usize,
    pub node_type: NodeType,
}

impl DomNode {
    /// Create an element node.
    pub fn element(tag: &str) -> Self {
        Self {
            id: 0,
            node_type: NodeType::Element {
                tag: tag.to_string(),
                attributes: HashMap::new(),
                children: Vec::new(),
            },
        }
    }

    /// Create a text node.
    pub fn text(content: &str) -> Self {
        Self {
            id: 0,
            node_type: NodeType::Text(content.to_string()),
        }
    }

    /// Check if this is an element node.
    pub fn is_element(&self) -> bool {
        matches!(self.node_type, NodeType::Element { .. })
    }

    /// Check if this is a text node.
    pub fn is_text(&self) -> bool {
        matches!(self.node_type, NodeType::Text(_))
    }

    /// Get the tag name (for elements).
    pub fn tag_name(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element { tag, .. } => Some(tag),
            _ => None,
        }
    }

    /// Get text content.
    pub fn text_content(&self) -> String {
        match &self.node_type {
            NodeType::Text(text) => text.clone(),
            NodeType::Element { children, .. } => {
                children.iter().map(|c| c.text_content()).collect()
            }
            NodeType::Comment(_) => String::new(),
        }
    }

    /// Get an attribute value.
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        match &self.node_type {
            NodeType::Element { attributes, .. } => attributes.get(name),
            _ => None,
        }
    }

    /// Set an attribute.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        if let NodeType::Element { attributes, .. } = &mut self.node_type {
            attributes.insert(name.to_string(), value.to_string());
        }
    }

    /// Remove an attribute.
    pub fn remove_attribute(&mut self, name: &str) {
        if let NodeType::Element { attributes, .. } = &mut self.node_type {
            attributes.remove(name);
        }
    }

    /// Check if has an attribute.
    pub fn has_attribute(&self, name: &str) -> bool {
        self.get_attribute(name).is_some()
    }

    /// Get classes.
    pub fn class_list(&self) -> Vec<String> {
        self.get_attribute("class")
            .map(|c| c.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }

    /// Check if has a class.
    pub fn has_class(&self, class: &str) -> bool {
        self.class_list().contains(&class.to_string())
    }

    /// Add a class.
    pub fn add_class(&mut self, class: &str) {
        let mut classes = self.class_list();
        if !classes.contains(&class.to_string()) {
            classes.push(class.to_string());
            self.set_attribute("class", &classes.join(" "));
        }
    }

    /// Remove a class.
    pub fn remove_class(&mut self, class: &str) {
        let classes: Vec<_> = self
            .class_list()
            .into_iter()
            .filter(|c| c != class)
            .collect();
        self.set_attribute("class", &classes.join(" "));
    }

    /// Get children.
    pub fn children(&self) -> &[DomNode] {
        match &self.node_type {
            NodeType::Element { children, .. } => children,
            _ => &[],
        }
    }

    /// Get mutable children.
    pub fn children_mut(&mut self) -> Option<&mut Vec<DomNode>> {
        match &mut self.node_type {
            NodeType::Element { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Append a child node.
    pub fn append_child(&mut self, child: DomNode) {
        if let Some(children) = self.children_mut() {
            children.push(child);
        }
    }

    /// Remove a child by index.
    pub fn remove_child(&mut self, index: usize) -> Option<DomNode> {
        self.children_mut().and_then(|children| {
            if index < children.len() {
                Some(children.remove(index))
            } else {
                None
            }
        })
    }

    /// Query for descendant elements.
    pub fn query_selector(&self, selector: &str) -> Vec<&DomNode> {
        let mut results = Vec::new();
        self.query_recursive(selector, &mut results);
        results
    }

    fn query_recursive<'a>(&'a self, selector: &str, results: &mut Vec<&'a DomNode>) {
        if self.matches_selector(selector) {
            results.push(self);
        }

        for child in self.children() {
            child.query_recursive(selector, results);
        }
    }

    /// Check if element matches a selector.
    pub fn matches_selector(&self, selector: &str) -> bool {
        let selector = selector.trim();

        // ID selector
        if let Some(id) = selector.strip_prefix('#') {
            return self.get_attribute("id") == Some(&id.to_string());
        }

        // Class selector
        if let Some(class) = selector.strip_prefix('.') {
            return self.has_class(class);
        }

        // Attribute selector
        if selector.starts_with('[') && selector.ends_with(']') {
            let attr = &selector[1..selector.len() - 1];
            if let Some((name, value)) = attr.split_once('=') {
                let value = value.trim_matches('"').trim_matches('\'');
                return self.get_attribute(name) == Some(&value.to_string());
            } else {
                return self.has_attribute(attr);
            }
        }

        // Tag selector
        self.tag_name() == Some(selector)
    }

    /// Convert to HTML string.
    pub fn to_html(&self) -> String {
        match &self.node_type {
            NodeType::Text(text) => text.clone(),
            NodeType::Comment(text) => format!("<!--{}-->", text),
            NodeType::Element {
                tag,
                attributes,
                children,
            } => {
                let attrs: String = attributes
                    .iter()
                    .map(|(k, v)| format!(" {}=\"{}\"", k, v))
                    .collect();

                let children_html: String = children.iter().map(|c| c.to_html()).collect();

                if children.is_empty() && is_void_element(tag) {
                    format!("<{}{} />", tag, attrs)
                } else {
                    format!("<{}{}>{}</{}>", tag, attrs, children_html, tag)
                }
            }
        }
    }
}

impl fmt::Debug for DomNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_html())
    }
}

/// Node type enumeration.
#[derive(Clone)]
pub enum NodeType {
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<DomNode>,
    },
    Text(String),
    Comment(String),
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Parse simple HTML string into DOM nodes.
fn parse_html(html: &str) -> DomNode {
    // This is a very simplified parser for testing purposes
    let html = html.trim();

    if html.starts_with('<') {
        // Parse element
        let end_tag = html.find('>').unwrap_or(html.len());
        let tag_content = &html[1..end_tag];

        // Handle self-closing tags
        let is_self_closing = tag_content.ends_with('/');
        let tag_content = tag_content.trim_end_matches('/').trim();

        // Split tag name and attributes
        let parts: Vec<&str> = tag_content.split_whitespace().collect();
        let tag = parts.first().copied().unwrap_or("div");

        let mut node = DomNode::element(tag);

        // Parse attributes
        for part in &parts[1..] {
            if let Some((key, value)) = part.split_once('=') {
                let value = value.trim_matches('"').trim_matches('\'');
                node.set_attribute(key, value);
            }
        }

        // Parse children if not self-closing
        if !is_self_closing && !is_void_element(tag) {
            let close_tag = format!("</{}>", tag);
            if let Some(close_pos) = html.rfind(&close_tag) {
                let inner_html = &html[end_tag + 1..close_pos];
                if !inner_html.is_empty() {
                    // Simplified: just add as text
                    if inner_html.contains('<') {
                        // Has child elements - parse recursively
                        let child = parse_html(inner_html);
                        node.append_child(child);
                    } else {
                        node.append_child(DomNode::text(inner_html));
                    }
                }
            }
        }

        node
    } else {
        // Text node
        DomNode::text(html)
    }
}

/// Builder for creating test DOM structures.
pub struct DomBuilder {
    node: DomNode,
}

impl DomBuilder {
    /// Create a new element builder.
    pub fn element(tag: &str) -> Self {
        Self {
            node: DomNode::element(tag),
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.node.set_attribute(name, value);
        self
    }

    /// Add a class.
    pub fn class(mut self, class: &str) -> Self {
        self.node.add_class(class);
        self
    }

    /// Add an ID.
    pub fn id(self, id: &str) -> Self {
        self.attr("id", id)
    }

    /// Set text content.
    pub fn text(mut self, text: &str) -> Self {
        self.node.append_child(DomNode::text(text));
        self
    }

    /// Add a child element.
    pub fn child(mut self, child: DomNode) -> Self {
        self.node.append_child(child);
        self
    }

    /// Add a child element from builder.
    pub fn child_builder(self, builder: DomBuilder) -> Self {
        self.child(builder.build())
    }

    /// Build the DOM node.
    pub fn build(self) -> DomNode {
        self.node
    }
}

/// Event simulation for testing.
#[derive(Debug, Clone)]
pub struct TestEvent {
    pub event_type: String,
    pub target_id: Option<usize>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub prevented: bool,
    pub stopped: bool,
    pub data: HashMap<String, String>,
}

impl TestEvent {
    /// Create a click event.
    pub fn click() -> Self {
        Self {
            event_type: "click".to_string(),
            target_id: None,
            bubbles: true,
            cancelable: true,
            prevented: false,
            stopped: false,
            data: HashMap::new(),
        }
    }

    /// Create an input event.
    pub fn input(value: &str) -> Self {
        let mut event = Self {
            event_type: "input".to_string(),
            target_id: None,
            bubbles: true,
            cancelable: false,
            prevented: false,
            stopped: false,
            data: HashMap::new(),
        };
        event.data.insert("value".to_string(), value.to_string());
        event
    }

    /// Create a change event.
    pub fn change(value: &str) -> Self {
        let mut event = Self {
            event_type: "change".to_string(),
            target_id: None,
            bubbles: true,
            cancelable: false,
            prevented: false,
            stopped: false,
            data: HashMap::new(),
        };
        event.data.insert("value".to_string(), value.to_string());
        event
    }

    /// Create a keydown event.
    pub fn keydown(key: &str) -> Self {
        let mut event = Self {
            event_type: "keydown".to_string(),
            target_id: None,
            bubbles: true,
            cancelable: true,
            prevented: false,
            stopped: false,
            data: HashMap::new(),
        };
        event.data.insert("key".to_string(), key.to_string());
        event
    }

    /// Create a submit event.
    pub fn submit() -> Self {
        Self {
            event_type: "submit".to_string(),
            target_id: None,
            bubbles: true,
            cancelable: true,
            prevented: false,
            stopped: false,
            data: HashMap::new(),
        }
    }

    /// Create a custom event.
    pub fn custom(event_type: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            target_id: None,
            bubbles: false,
            cancelable: false,
            prevented: false,
            stopped: false,
            data: HashMap::new(),
        }
    }

    /// Set the target element.
    pub fn target(mut self, id: usize) -> Self {
        self.target_id = Some(id);
        self
    }

    /// Add event data.
    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    /// Prevent default.
    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.prevented = true;
        }
    }

    /// Stop propagation.
    pub fn stop_propagation(&mut self) {
        self.stopped = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_node_element() {
        let node = DomNode::element("div");
        assert!(node.is_element());
        assert_eq!(node.tag_name(), Some("div"));
    }

    #[test]
    fn test_dom_node_attributes() {
        let mut node = DomNode::element("input");
        node.set_attribute("type", "text");
        node.set_attribute("value", "hello");

        assert_eq!(node.get_attribute("type"), Some(&"text".to_string()));
        assert_eq!(node.get_attribute("value"), Some(&"hello".to_string()));
    }

    #[test]
    fn test_dom_node_classes() {
        let mut node = DomNode::element("div");
        node.add_class("container");
        node.add_class("active");

        assert!(node.has_class("container"));
        assert!(node.has_class("active"));

        node.remove_class("active");
        assert!(!node.has_class("active"));
    }

    #[test]
    fn test_dom_node_children() {
        let mut parent = DomNode::element("ul");
        parent.append_child(DomNode::element("li"));
        parent.append_child(DomNode::element("li"));

        assert_eq!(parent.children().len(), 2);
    }

    #[test]
    fn test_dom_builder() {
        let node = DomBuilder::element("div")
            .id("main")
            .class("container")
            .text("Hello")
            .build();

        assert_eq!(node.get_attribute("id"), Some(&"main".to_string()));
        assert!(node.has_class("container"));
    }

    #[test]
    fn test_query_selector() {
        let node = DomBuilder::element("div")
            .id("root")
            .child(
                DomBuilder::element("span")
                    .class("label")
                    .build(),
            )
            .build();

        assert!(!node.query_selector(".label").is_empty());
    }

    #[test]
    fn test_to_html() {
        let node = DomBuilder::element("div")
            .class("test")
            .text("Hello")
            .build();

        let html = node.to_html();
        assert!(html.contains("<div"));
        assert!(html.contains("class=\"test\""));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_event() {
        let event = TestEvent::click();
        assert_eq!(event.event_type, "click");
        assert!(event.bubbles);
    }
}

