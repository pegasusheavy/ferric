//! Template compiler that generates DOM operations from the AST.

use super::{TemplateNode, ElementNode, TextNode, InterpolationNode, BindingType};
use super::context::TemplateContext;
use super::instance::{TemplateInstance, TemplateRenderer};
use wasm_bindgen::prelude::*;

/// Compiled template that can be instantiated.
pub struct CompiledTemplate {
    /// The root nodes of the template.
    nodes: Vec<TemplateNode>,
    /// The original template string (for debugging).
    #[allow(dead_code)]
    source: String,
}

impl CompiledTemplate {
    /// Create a new compiled template from parsed nodes.
    pub fn new(nodes: Vec<TemplateNode>) -> Self {
        Self {
            nodes,
            source: String::new(),
        }
    }

    /// Create from source string.
    pub fn from_source(source: &str, nodes: Vec<TemplateNode>) -> Self {
        Self {
            nodes,
            source: source.to_string(),
        }
    }

    /// Get the template nodes.
    pub fn nodes(&self) -> &[TemplateNode] {
        &self.nodes
    }

    /// Create a DOM fragment from this template (legacy API).
    pub fn create_fragment(&self, document: &web_sys::Document) -> Result<web_sys::DocumentFragment, JsValue> {
        let fragment = document.create_document_fragment();

        for node in &self.nodes {
            let dom_node = self.create_node(document, node)?;
            fragment.append_child(&dom_node)?;
        }

        Ok(fragment)
    }

    /// Render the template with a context, creating a reactive instance.
    pub fn render(&self, context: TemplateContext) -> Result<TemplateInstance, JsValue> {
        TemplateRenderer::new(context).render(&self.nodes)
    }

    /// Render with an empty context.
    pub fn render_static(&self) -> Result<TemplateInstance, JsValue> {
        self.render(TemplateContext::new())
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

        // Set up bindings
        for binding in &el.bindings {
            match binding.binding_type {
                BindingType::Property => {
                    // Store binding info as data attribute for later processing
                    element.set_attribute(
                        &format!("data-bind-prop-{}", binding.target),
                        &binding.expression
                    )?;
                }
                BindingType::Event => {
                    element.set_attribute(
                        &format!("data-bind-event-{}", binding.target),
                        &binding.expression
                    )?;
                }
                BindingType::TwoWay => {
                    element.set_attribute(
                        &format!("data-bind-model-{}", binding.target),
                        &binding.expression
                    )?;
                }
                BindingType::Attribute => {
                    element.set_attribute(
                        &format!("data-bind-attr-{}", binding.target),
                        &binding.expression
                    )?;
                }
                BindingType::Class => {
                    element.set_attribute(
                        &format!("data-bind-class-{}", binding.target),
                        &binding.expression
                    )?;
                }
                BindingType::Style => {
                    element.set_attribute(
                        &format!("data-bind-style-{}", binding.target),
                        &binding.expression
                    )?;
                }
                BindingType::Interpolation => {
                    // Interpolation is handled at text node level
                }
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
        // Create a text node with a placeholder
        let text_node = document.create_text_node(&format!("{{{{{}}}}}", interp.expression));
        Ok(text_node.into())
    }
}

/// Compile a template string into an executable template.
pub fn compile_template(template: &str) -> Result<CompiledTemplate, super::ParseError> {
    let nodes = super::parse_template(template)?;
    Ok(CompiledTemplate::from_source(template, nodes))
}

/// Compile and render a template in one step.
pub fn render(template: &str, context: TemplateContext) -> Result<TemplateInstance, String> {
    let compiled = compile_template(template).map_err(|e| e.to_string())?;
    compiled.render(context).map_err(|e| format!("{:?}", e))
}

/// Compile and render a template with no context.
pub fn render_static(template: &str) -> Result<TemplateInstance, String> {
    render(template, TemplateContext::new())
}

/// A builder for creating templates with context.
pub struct TemplateBuilder {
    template: String,
    context: TemplateContext,
}

impl TemplateBuilder {
    /// Create a new template builder.
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
            context: TemplateContext::new(),
        }
    }

    /// Add a value to the context.
    pub fn with<T: super::context::ContextValue + 'static>(self, key: &str, value: T) -> Self {
        self.context.set(key, value);
        self
    }

    /// Build and render the template.
    pub fn build(self) -> Result<TemplateInstance, String> {
        render(&self.template, self.context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_template() {
        let template = "<div>Hello World</div>";
        let compiled = compile_template(template);
        assert!(compiled.is_ok());

        let compiled = compiled.unwrap();
        assert_eq!(compiled.nodes().len(), 1);
    }

    #[test]
    fn test_compile_with_interpolation() {
        let template = "<div>Hello {{ name }}</div>";
        let compiled = compile_template(template);
        assert!(compiled.is_ok());
    }

    #[test]
    fn test_compile_with_binding() {
        let template = r#"<input [value]="name" />"#;
        let compiled = compile_template(template);
        assert!(compiled.is_ok());
    }

    #[test]
    fn test_compile_with_event() {
        let template = r#"<button (click)="onClick()">Click me</button>"#;
        let compiled = compile_template(template);
        assert!(compiled.is_ok());
    }
}
