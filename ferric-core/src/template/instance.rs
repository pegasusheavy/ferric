//! Template instance - a rendered template with active bindings.

use super::context::TemplateContext;
use super::expression::{evaluate, evaluate_to_bool, evaluate_to_string};
use super::{Binding, BindingType, TemplateNode, ElementNode, TextNode, InterpolationNode};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Node, Text, HtmlElement, HtmlInputElement};
use std::cell::RefCell;
use std::rc::Rc;

/// An active binding that updates the DOM when context changes.
struct ActiveBinding {
    /// The type of binding.
    binding_type: ActiveBindingType,
    /// The expression to evaluate.
    expression: String,
    /// Cleanup function.
    cleanup: Option<Box<dyn Fn()>>,
}

enum ActiveBindingType {
    /// Text interpolation.
    Text(Text),
    /// Property binding on an element.
    Property { element: Element, property: String },
    /// Attribute binding on an element.
    Attribute { element: Element, attribute: String },
    /// Class binding on an element.
    Class { element: Element, class_name: String },
    /// Style binding on an element.
    Style { element: Element, style_prop: String },
    /// Event binding (stores the closure).
    Event { element: Element, event_name: String },
}

/// A rendered template instance with active bindings.
pub struct TemplateInstance {
    /// The root element.
    root: Element,
    /// The template context.
    context: TemplateContext,
    /// Active bindings that update the DOM.
    bindings: Rc<RefCell<Vec<ActiveBinding>>>,
    /// Named element references (#ref).
    references: Rc<RefCell<rustc_hash::FxHashMap<String, Element>>>,
    /// Event closures that need to be kept alive.
    closures: Rc<RefCell<Vec<Closure<dyn Fn(web_sys::Event)>>>>,
}

impl TemplateInstance {
    /// Get the root element.
    pub fn root(&self) -> &Element {
        &self.root
    }

    /// Get the template context.
    pub fn context(&self) -> &TemplateContext {
        &self.context
    }

    /// Get a reference by name.
    pub fn get_ref(&self, name: &str) -> Option<Element> {
        self.references.borrow().get(name).cloned()
    }

    /// Update the template with new context values.
    pub fn update(&self) {
        for binding in self.bindings.borrow().iter() {
            self.update_binding(binding);
        }
    }

    /// Update a single binding.
    fn update_binding(&self, binding: &ActiveBinding) {
        let value = evaluate(&binding.expression, &self.context);

        match &binding.binding_type {
            ActiveBindingType::Text(text_node) => {
                text_node.set_data(&value.to_string());
            }
            ActiveBindingType::Property { element, property } => {
                self.set_property(element, property, &value.to_string());
            }
            ActiveBindingType::Attribute { element, attribute } => {
                if value.to_bool() {
                    let _ = element.set_attribute(attribute, &value.to_string());
                } else {
                    let _ = element.remove_attribute(attribute);
                }
            }
            ActiveBindingType::Class { element, class_name } => {
                if value.to_bool() {
                    let _ = element.class_list().add_1(class_name);
                } else {
                    let _ = element.class_list().remove_1(class_name);
                }
            }
            ActiveBindingType::Style { element, style_prop } => {
                if let Some(html_el) = element.dyn_ref::<HtmlElement>() {
                    let _ = html_el.style().set_property(style_prop, &value.to_string());
                }
            }
            ActiveBindingType::Event { .. } => {
                // Events don't need updating
            }
        }
    }

    /// Set a DOM property.
    fn set_property(&self, element: &Element, property: &str, value: &str) {
        match property {
            "value" => {
                if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                    input.set_value(value);
                }
            }
            "checked" => {
                if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                    input.set_checked(value == "true" || !value.is_empty());
                }
            }
            "disabled" => {
                if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                    input.set_disabled(value == "true" || !value.is_empty());
                }
            }
            "textContent" => {
                element.set_text_content(Some(value));
            }
            "innerHTML" => {
                element.set_inner_html(value);
            }
            _ => {
                // Fall back to attribute
                let _ = element.set_attribute(property, value);
            }
        }
    }

    /// Destroy the instance and clean up bindings.
    pub fn destroy(self) {
        for binding in self.bindings.borrow().iter() {
            if let Some(ref cleanup) = binding.cleanup {
                cleanup();
            }
        }
        // Drop closures to clean up event listeners
        drop(self.closures);
    }
}

/// Builder for creating template instances.
pub struct TemplateRenderer {
    document: Document,
    context: TemplateContext,
    bindings: Vec<ActiveBinding>,
    references: rustc_hash::FxHashMap<String, Element>,
    closures: Vec<Closure<dyn Fn(web_sys::Event)>>,
}

impl TemplateRenderer {
    /// Create a new renderer.
    pub fn new(context: TemplateContext) -> Self {
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");

        Self {
            document,
            context,
            bindings: Vec::new(),
            references: rustc_hash::FxHashMap::default(),
            closures: Vec::new(),
        }
    }

    /// Render a template AST.
    pub fn render(mut self, nodes: &[TemplateNode]) -> Result<TemplateInstance, JsValue> {
        // Create a container element
        let container = self.document.create_element("div")?;

        // Render all nodes
        for node in nodes {
            let dom_node = self.render_node(node)?;
            container.append_child(&dom_node)?;
        }

        // If there's only one child, use it as the root
        let root = if container.child_element_count() == 1 {
            container.first_element_child().unwrap_or(container)
        } else {
            container
        };

        Ok(TemplateInstance {
            root,
            context: self.context,
            bindings: Rc::new(RefCell::new(self.bindings)),
            references: Rc::new(RefCell::new(self.references)),
            closures: Rc::new(RefCell::new(self.closures)),
        })
    }

    /// Render a single template node.
    fn render_node(&mut self, node: &TemplateNode) -> Result<Node, JsValue> {
        match node {
            TemplateNode::Element(el) => self.render_element(el),
            TemplateNode::Text(text) => self.render_text(text),
            TemplateNode::Interpolation(interp) => self.render_interpolation(interp),
            TemplateNode::Component(comp) => {
                // Render components as custom elements for now
                let el = ElementNode {
                    tag: comp.selector.clone(),
                    attributes: Vec::new(),
                    bindings: Vec::new(),
                    children: comp.children.clone(),
                    reference: comp.reference.clone(),
                    structural_directive: None,
                };
                self.render_element(&el)
            }
        }
    }

    /// Render an element node.
    fn render_element(&mut self, el: &ElementNode) -> Result<Node, JsValue> {
        // Handle structural directives
        if let Some(ref directive) = el.structural_directive {
            return self.render_structural_directive(el, directive);
        }

        let element = self.document.create_element(&el.tag)?;

        // Set static attributes
        for (name, value) in &el.attributes {
            element.set_attribute(name, value)?;
        }

        // Process bindings
        for binding in &el.bindings {
            self.process_binding(&element, binding)?;
        }

        // Store reference
        if let Some(ref name) = el.reference {
            self.references.insert(name.clone(), element.clone());
        }

        // Render children
        for child in &el.children {
            let child_node = self.render_node(child)?;
            element.append_child(&child_node)?;
        }

        Ok(element.into())
    }

    /// Render a text node.
    fn render_text(&mut self, text: &TextNode) -> Result<Node, JsValue> {
        Ok(self.document.create_text_node(&text.content).into())
    }

    /// Render an interpolation node.
    fn render_interpolation(&mut self, interp: &InterpolationNode) -> Result<Node, JsValue> {
        let value = evaluate_to_string(&interp.expression, &self.context);
        let text_node = self.document.create_text_node(&value);

        // Add binding for reactive updates
        self.bindings.push(ActiveBinding {
            binding_type: ActiveBindingType::Text(text_node.clone()),
            expression: interp.expression.clone(),
            cleanup: None,
        });

        Ok(text_node.into())
    }

    /// Process a binding on an element.
    fn process_binding(&mut self, element: &Element, binding: &Binding) -> Result<(), JsValue> {
        match binding.binding_type {
            BindingType::Property => {
                self.process_property_binding(element, binding)?;
            }
            BindingType::Event => {
                self.process_event_binding(element, binding)?;
            }
            BindingType::TwoWay => {
                self.process_two_way_binding(element, binding)?;
            }
            BindingType::Attribute => {
                self.process_attribute_binding(element, binding)?;
            }
            BindingType::Class => {
                self.process_class_binding(element, binding)?;
            }
            BindingType::Style => {
                self.process_style_binding(element, binding)?;
            }
            BindingType::Interpolation => {
                // Interpolation is handled differently
            }
        }

        Ok(())
    }

    /// Process a property binding [property]="expression".
    fn process_property_binding(&mut self, element: &Element, binding: &Binding) -> Result<(), JsValue> {
        let target = &binding.target;

        // Check for special prefixes
        if let Some(attr) = target.strip_prefix("attr.") {
            return self.process_attribute_binding_with_name(element, attr, &binding.expression);
        }

        if let Some(class_name) = target.strip_prefix("class.") {
            return self.process_class_binding_with_name(element, class_name, &binding.expression);
        }

        if let Some(style_prop) = target.strip_prefix("style.") {
            return self.process_style_binding_with_name(element, style_prop, &binding.expression);
        }

        // Regular property binding
        let value = evaluate_to_string(&binding.expression, &self.context);
        self.set_element_property(element, target, &value)?;

        self.bindings.push(ActiveBinding {
            binding_type: ActiveBindingType::Property {
                element: element.clone(),
                property: target.clone(),
            },
            expression: binding.expression.clone(),
            cleanup: None,
        });

        Ok(())
    }

    /// Process an event binding (event)="handler".
    fn process_event_binding(&mut self, element: &Element, binding: &Binding) -> Result<(), JsValue> {
        let context = self.context.clone();
        let handler_expr = binding.expression.clone();
        let element_clone = element.clone();

        // Create event handler closure
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            // Set $event in context for handler
            context.set("$event", format!("{:?}", event));

            // Evaluate the handler expression
            // In a real implementation, this would invoke a method on the component
            let _ = evaluate(&handler_expr, &context);
        }) as Box<dyn Fn(web_sys::Event)>);

        element.add_event_listener_with_callback(
            &binding.target,
            closure.as_ref().unchecked_ref()
        )?;

        // Store closure to keep it alive
        self.closures.push(closure);

        self.bindings.push(ActiveBinding {
            binding_type: ActiveBindingType::Event {
                element: element_clone,
                event_name: binding.target.clone(),
            },
            expression: binding.expression.clone(),
            cleanup: None,
        });

        Ok(())
    }

    /// Process a two-way binding [(property)]="value".
    fn process_two_way_binding(&mut self, element: &Element, binding: &Binding) -> Result<(), JsValue> {
        // Set up property binding
        self.process_property_binding(element, &Binding {
            binding_type: BindingType::Property,
            target: binding.target.clone(),
            expression: binding.expression.clone(),
        })?;

        // Set up event binding for input events
        let event_name = match binding.target.as_str() {
            "value" => "input",
            "checked" => "change",
            _ => "change",
        };

        let context = self.context.clone();
        let property = binding.target.clone();
        let expr = binding.expression.clone();
        let element_clone = element.clone();

        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            // Get the new value from the element
            let new_value = if let Some(input) = element_clone.dyn_ref::<HtmlInputElement>() {
                match property.as_str() {
                    "checked" => input.checked().to_string(),
                    _ => input.value(),
                }
            } else {
                element_clone.text_content().unwrap_or_default()
            };

            // Update the context
            context.update(&expr, new_value);
        }) as Box<dyn Fn(web_sys::Event)>);

        element.add_event_listener_with_callback(
            event_name,
            closure.as_ref().unchecked_ref()
        )?;

        self.closures.push(closure);

        Ok(())
    }

    /// Process an attribute binding.
    fn process_attribute_binding(&mut self, element: &Element, binding: &Binding) -> Result<(), JsValue> {
        self.process_attribute_binding_with_name(element, &binding.target, &binding.expression)
    }

    fn process_attribute_binding_with_name(&mut self, element: &Element, attr: &str, expr: &str) -> Result<(), JsValue> {
        let value = evaluate(expr, &self.context);

        if value.to_bool() {
            element.set_attribute(attr, &value.to_string())?;
        }

        self.bindings.push(ActiveBinding {
            binding_type: ActiveBindingType::Attribute {
                element: element.clone(),
                attribute: attr.to_string(),
            },
            expression: expr.to_string(),
            cleanup: None,
        });

        Ok(())
    }

    /// Process a class binding.
    fn process_class_binding(&mut self, element: &Element, binding: &Binding) -> Result<(), JsValue> {
        self.process_class_binding_with_name(element, &binding.target, &binding.expression)
    }

    fn process_class_binding_with_name(&mut self, element: &Element, class_name: &str, expr: &str) -> Result<(), JsValue> {
        let value = evaluate_to_bool(expr, &self.context);

        if value {
            element.class_list().add_1(class_name)?;
        }

        self.bindings.push(ActiveBinding {
            binding_type: ActiveBindingType::Class {
                element: element.clone(),
                class_name: class_name.to_string(),
            },
            expression: expr.to_string(),
            cleanup: None,
        });

        Ok(())
    }

    /// Process a style binding.
    fn process_style_binding(&mut self, element: &Element, binding: &Binding) -> Result<(), JsValue> {
        self.process_style_binding_with_name(element, &binding.target, &binding.expression)
    }

    fn process_style_binding_with_name(&mut self, element: &Element, style_prop: &str, expr: &str) -> Result<(), JsValue> {
        let value = evaluate_to_string(expr, &self.context);

        if let Some(html_el) = element.dyn_ref::<HtmlElement>() {
            html_el.style().set_property(style_prop, &value)?;
        }

        self.bindings.push(ActiveBinding {
            binding_type: ActiveBindingType::Style {
                element: element.clone(),
                style_prop: style_prop.to_string(),
            },
            expression: expr.to_string(),
            cleanup: None,
        });

        Ok(())
    }

    /// Set a property on an element.
    fn set_element_property(&self, element: &Element, property: &str, value: &str) -> Result<(), JsValue> {
        match property {
            "value" => {
                if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                    input.set_value(value);
                }
            }
            "checked" => {
                if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                    input.set_checked(value == "true" || !value.is_empty());
                }
            }
            "disabled" => {
                if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                    input.set_disabled(value == "true" || !value.is_empty());
                }
            }
            "textContent" => {
                element.set_text_content(Some(value));
            }
            "innerHTML" => {
                element.set_inner_html(value);
            }
            _ => {
                element.set_attribute(property, value)?;
            }
        }
        Ok(())
    }

    /// Render a structural directive (*ngIf, *ngFor, etc.).
    fn render_structural_directive(
        &mut self,
        el: &ElementNode,
        directive: &super::StructuralDirective,
    ) -> Result<Node, JsValue> {
        match directive.name.as_str() {
            "if" | "ngIf" => self.render_if_directive(el, &directive.expression),
            "for" | "ngFor" => self.render_for_directive(el, &directive.expression),
            _ => {
                // Unknown directive - render element normally
                self.render_element(&ElementNode {
                    structural_directive: None,
                    ..el.clone()
                })
            }
        }
    }

    /// Render *ngIf directive.
    fn render_if_directive(&mut self, el: &ElementNode, expr: &str) -> Result<Node, JsValue> {
        let condition = evaluate_to_bool(expr, &self.context);

        if condition {
            // Render the element without the directive
            self.render_element(&ElementNode {
                structural_directive: None,
                ..el.clone()
            })
        } else {
            // Render a comment placeholder
            Ok(self.document.create_comment(&format!("ngIf: {}", expr)).into())
        }
    }

    /// Render *ngFor directive.
    fn render_for_directive(&mut self, el: &ElementNode, expr: &str) -> Result<Node, JsValue> {
        // Parse "let item of items" or "item of items"
        let parts: Vec<&str> = expr.split(" of ").collect();

        if parts.len() != 2 {
            return Ok(self.document.create_comment(&format!("Invalid ngFor: {}", expr)).into());
        }

        let item_name = parts[0].trim().trim_start_matches("let ");
        let items_expr = parts[1].trim();

        // Create a document fragment to hold all items
        let fragment = self.document.create_document_fragment();

        // Get the items array from context
        // For now, we'll use a simple comma-separated string
        if let Some(items_ref) = self.context.get(items_expr) {
            let items_str = items_ref.to_string();
            let items: Vec<&str> = items_str.split(',').map(|s| s.trim()).collect();

            for (index, item) in items.iter().enumerate() {
                // Create child context
                let child_ctx = self.context.child();
                child_ctx.set(item_name, item.to_string());
                child_ctx.set("$index", index as i32);
                child_ctx.set("$first", index == 0);
                child_ctx.set("$last", index == items.len() - 1);
                child_ctx.set("$even", index % 2 == 0);
                child_ctx.set("$odd", index % 2 == 1);

                // Render element with child context
                let mut child_renderer = TemplateRenderer {
                    document: self.document.clone(),
                    context: child_ctx,
                    bindings: Vec::new(),
                    references: rustc_hash::FxHashMap::default(),
                    closures: Vec::new(),
                };

                let node = child_renderer.render_element(&ElementNode {
                    structural_directive: None,
                    ..el.clone()
                })?;

                fragment.append_child(&node)?;

                // Transfer bindings and closures
                self.bindings.extend(child_renderer.bindings);
                self.closures.extend(child_renderer.closures);
            }
        }

        Ok(fragment.into())
    }
}

/// Render a template string with a context.
pub fn render_template(template: &str, context: TemplateContext) -> Result<TemplateInstance, String> {
    let nodes = super::parse_template(template).map_err(|e| e.to_string())?;

    TemplateRenderer::new(context)
        .render(&nodes)
        .map_err(|e| format!("{:?}", e))
}

