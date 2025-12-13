//! Template compiler that generates DOM operations from the AST.

use super::{TemplateNode, ElementNode, TextNode, InterpolationNode, BindingType};
use wasm_bindgen::prelude::*;

/// Compiled template that can be instantiated.
pub struct CompiledTemplate {
    /// The root nodes of the template.
    nodes: Vec<TemplateNode>,
}

impl CompiledTemplate {
    /// Create a new compiled template from parsed nodes.
    pub fn new(nodes: Vec<TemplateNode>) -> Self {
        Self { nodes }
    }

    /// Create a DOM fragment from this template.
    pub fn create_fragment(&self, document: &web_sys::Document) -> Result<web_sys::DocumentFragment, JsValue> {
        let fragment = document.create_document_fragment();

        for node in &self.nodes {
            let dom_node = self.create_node(document, node)?;
            fragment.append_child(&dom_node)?;
        }

        Ok(fragment)
    }

    fn create_node(&self, document: &web_sys::Document, node: &TemplateNode) -> Result<web_sys::Node, JsValue> {
        match node {
            TemplateNode::Element(el) => self.create_element(document, el),
            TemplateNode::Text(text) => self.create_text(document, text),
            TemplateNode::Interpolation(interp) => self.create_interpolation(document, interp),
            TemplateNode::Component(comp) => {
                // Components are handled specially
                self.create_element(document, &ElementNode {
                    tag: comp.selector.clone(),
                    attributes: Vec::new(),
                    bindings: Vec::new(),
                    children: comp.children.clone(),
                    reference: comp.reference.clone(),
                    structural_directive: None,
                })
            }
        }
    }

    fn create_element(&self, document: &web_sys::Document, el: &ElementNode) -> Result<web_sys::Node, JsValue> {
        let element = document.create_element(&el.tag)?;

        // Set static attributes
        for (name, value) in &el.attributes {
            element.set_attribute(name, value)?;
        }

        // Set up bindings (placeholder - actual binding logic would be more complex)
        for binding in &el.bindings {
            match binding.binding_type {
                BindingType::Property => {
                    // Property binding would set up reactive updates
                    element.set_attribute(&format!("data-bind-{}", binding.target), &binding.expression)?;
                }
                BindingType::Event => {
                    // Event binding would attach event listeners
                    element.set_attribute(&format!("data-event-{}", binding.target), &binding.expression)?;
                }
                BindingType::TwoWay => {
                    // Two-way binding combines property and event
                    element.set_attribute(&format!("data-model-{}", binding.target), &binding.expression)?;
                }
                _ => {}
            }
        }

        // Create children
        for child in &el.children {
            let child_node = self.create_node(document, child)?;
            element.append_child(&child_node)?;
        }

        Ok(element.into())
    }

    fn create_text(&self, document: &web_sys::Document, text: &TextNode) -> Result<web_sys::Node, JsValue> {
        Ok(document.create_text_node(&text.content).into())
    }

    fn create_interpolation(&self, document: &web_sys::Document, interp: &InterpolationNode) -> Result<web_sys::Node, JsValue> {
        // Create a text node that will be updated reactively
        let text_node = document.create_text_node(&format!("{{{{ {} }}}}", interp.expression));
        // In a real implementation, this would set up reactive binding
        Ok(text_node.into())
    }
}

/// Compile a template string into an executable template.
pub fn compile_template(template: &str) -> Result<CompiledTemplate, super::ParseError> {
    let nodes = super::parse_template(template)?;
    Ok(CompiledTemplate::new(nodes))
}

