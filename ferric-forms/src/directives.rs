//! Form directives for template-driven forms.
//!
//! Provides directives for binding form controls to template elements.
//!
//! ## Directives
//!
//! - `feModel` - Two-way binding directive
//! - `feFormGroup` - Bind a form group to an element
//! - `feFormControlName` - Bind a named control within a group
//! - `feFormArrayName` - Bind a form array by name
//!
//! ## Usage
//!
//! ```html
//! <form [feFormGroup]="form">
//!   <input feFormControlName="email" />
//!   <input feFormControlName="password" />
//!
//!   <div feFormArrayName="items">
//!     <div *feFor="let item of form.controls['items'].controls; index as i">
//!       <input [feFormControl]="item" />
//!     </div>
//!   </div>
//! </form>
//! ```

use crate::controls::{AbstractControl, FormArray, FormControl, FormGroup};
use crate::binding::UpdateOn;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Directive for two-way model binding.
///
/// Used like: `<input [(feModel)]="value" />`
pub struct FeModelDirective<T: Clone + 'static> {
    control: FormControl<T>,
    update_on: UpdateOn,
}

impl<T: Clone + 'static> FeModelDirective<T> {
    /// Create a new model directive.
    pub fn new(initial: T) -> Self {
        Self {
            control: FormControl::new(initial),
            update_on: UpdateOn::Input,
        }
    }

    /// Set when to update the model.
    pub fn update_on(mut self, update_on: UpdateOn) -> Self {
        self.update_on = update_on;
        self
    }

    /// Get the current value.
    pub fn value(&self) -> T {
        self.control.value()
    }

    /// Set the value.
    pub fn set_value(&self, value: T) {
        self.control.set_value(value);
    }

    /// Get the underlying control.
    pub fn control(&self) -> &FormControl<T> {
        &self.control
    }
}

/// Directive for binding form groups.
///
/// Used like: `<form [feFormGroup]="myForm">`
pub struct FeFormGroupDirective {
    group: Rc<FormGroup>,
    submitted: RefCell<bool>,
}

impl FeFormGroupDirective {
    /// Create a new form group directive.
    pub fn new(group: FormGroup) -> Self {
        Self {
            group: Rc::new(group),
            submitted: RefCell::new(false),
        }
    }

    /// Create from an existing Rc<FormGroup>.
    pub fn from_rc(group: Rc<FormGroup>) -> Self {
        Self {
            group,
            submitted: RefCell::new(false),
        }
    }

    /// Get the form group.
    pub fn form(&self) -> &FormGroup {
        &self.group
    }

    /// Check if the form was submitted.
    pub fn submitted(&self) -> bool {
        *self.submitted.borrow()
    }

    /// Handle form submit.
    pub fn on_submit(&self) -> bool {
        *self.submitted.borrow_mut() = true;
        use crate::controls::ControlContainer;
        self.group.mark_all_as_touched();
        self.group.valid()
    }

    /// Reset the form.
    pub fn reset(&self) {
        *self.submitted.borrow_mut() = false;
        self.group.reset();
    }

    /// Get a control by path (dot notation).
    pub fn get(&self, path: &str) -> Option<Rc<dyn AbstractControl>> {
        // Parse path and traverse
        let parts: Vec<&str> = path.split('.').collect();
        self.get_by_path(&parts)
    }

    fn get_by_path(&self, _parts: &[&str]) -> Option<Rc<dyn AbstractControl>> {
        // Simplified - would traverse the form tree
        None
    }
}

/// Directive for binding a named control within a form group.
///
/// Used like: `<input feFormControlName="email" />`
pub struct FeFormControlNameDirective {
    name: String,
    parent: Option<Rc<FeFormGroupDirective>>,
}

impl FeFormControlNameDirective {
    /// Create a new control name directive.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: None,
        }
    }

    /// Set the parent form group.
    pub fn set_parent(&mut self, parent: Rc<FeFormGroupDirective>) {
        self.parent = Some(parent);
    }

    /// Get the control name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the control from the parent.
    pub fn control(&self) -> Option<Rc<dyn AbstractControl>> {
        self.parent.as_ref().and_then(|p| p.get(&self.name))
    }
}

/// Directive for binding a form array.
///
/// Used like: `<div feFormArrayName="items">`
pub struct FeFormArrayNameDirective {
    name: String,
    parent: Option<Rc<FeFormGroupDirective>>,
}

impl FeFormArrayNameDirective {
    /// Create a new array name directive.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: None,
        }
    }

    /// Set the parent form group.
    pub fn set_parent(&mut self, parent: Rc<FeFormGroupDirective>) {
        self.parent = Some(parent);
    }

    /// Get the array name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Directive for binding a form control directly.
///
/// Used like: `<input [feFormControl]="myControl" />`
pub struct FeFormControlDirective<T: Clone + 'static> {
    control: Rc<FormControl<T>>,
}

impl<T: Clone + 'static> FeFormControlDirective<T> {
    /// Create a new control directive.
    pub fn new(control: FormControl<T>) -> Self {
        Self {
            control: Rc::new(control),
        }
    }

    /// Create from an existing Rc.
    pub fn from_rc(control: Rc<FormControl<T>>) -> Self {
        Self { control }
    }

    /// Get the control.
    pub fn control(&self) -> &FormControl<T> {
        &self.control
    }

    /// Get the value.
    pub fn value(&self) -> T {
        self.control.value()
    }

    /// Set the value.
    pub fn set_value(&self, value: T) {
        self.control.set_value(value);
    }
}

/// Container for form directives in a template.
pub struct FormDirectiveContainer {
    groups: HashMap<String, Rc<FeFormGroupDirective>>,
    controls: HashMap<String, Rc<dyn AbstractControl>>,
}

impl FormDirectiveContainer {
    /// Create a new container.
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            controls: HashMap::new(),
        }
    }

    /// Register a form group.
    pub fn register_group(&mut self, name: &str, directive: FeFormGroupDirective) {
        self.groups.insert(name.to_string(), Rc::new(directive));
    }

    /// Get a form group.
    pub fn get_group(&self, name: &str) -> Option<Rc<FeFormGroupDirective>> {
        self.groups.get(name).cloned()
    }

    /// Register a control.
    pub fn register_control(&mut self, name: &str, control: Rc<dyn AbstractControl>) {
        self.controls.insert(name.to_string(), control);
    }

    /// Get a control.
    pub fn get_control(&self, name: &str) -> Option<Rc<dyn AbstractControl>> {
        self.controls.get(name).cloned()
    }
}

impl Default for FormDirectiveContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for form behavior.
#[derive(Debug, Clone)]
pub struct FormConfig {
    /// When to update the model.
    pub update_on: UpdateOn,
    /// Whether to validate on submit only.
    pub validate_on_submit: bool,
    /// Whether to show errors only after submit.
    pub show_errors_on_submit: bool,
}

impl Default for FormConfig {
    fn default() -> Self {
        Self {
            update_on: UpdateOn::Input,
            validate_on_submit: false,
            show_errors_on_submit: false,
        }
    }
}

/// Directive attributes for parsing from templates.
#[derive(Debug, Clone)]
pub struct DirectiveAttributes {
    /// The directive name (e.g., "feFormGroup").
    pub directive: String,
    /// Bound value expression.
    pub value: Option<String>,
    /// Additional attributes.
    pub attrs: HashMap<String, String>,
}

impl DirectiveAttributes {
    /// Parse directive attributes from an element.
    pub fn parse(attrs: &HashMap<String, String>) -> Vec<Self> {
        let mut directives = Vec::new();

        for (key, value) in attrs {
            if key.starts_with("fe") || key.starts_with("[fe") || key.starts_with("[(fe") {
                directives.push(DirectiveAttributes {
                    directive: key.clone(),
                    value: Some(value.clone()),
                    attrs: attrs.clone(),
                });
            }
        }

        directives
    }

    /// Check if this is a two-way binding.
    pub fn is_two_way(&self) -> bool {
        self.directive.starts_with("[(") && self.directive.ends_with(")]")
    }

    /// Check if this is a property binding.
    pub fn is_property_binding(&self) -> bool {
        self.directive.starts_with('[') && !self.is_two_way()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fe_model_directive() {
        let directive = FeModelDirective::new("initial".to_string());
        assert_eq!(directive.value(), "initial");

        directive.set_value("updated".to_string());
        assert_eq!(directive.value(), "updated");
    }

    #[test]
    fn test_form_group_directive() {
        let group = FormGroup::new();
        let directive = FeFormGroupDirective::new(group);

        assert!(!directive.submitted());
    }

    #[test]
    fn test_directive_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert("feFormGroup".to_string(), "myForm".to_string());
        attrs.insert("class".to_string(), "form".to_string());

        let directives = DirectiveAttributes::parse(&attrs);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].directive, "feFormGroup");
    }
}
