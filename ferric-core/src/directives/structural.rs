//! Structural directives that modify DOM structure.

/// The *if directive - conditionally renders content.
pub struct IfDirective {
    condition: bool,
    template: Option<web_sys::Element>,
    view_container: Option<web_sys::Element>,
}

impl IfDirective {
    /// Create a new *if directive.
    pub fn new() -> Self {
        Self {
            condition: false,
            template: None,
            view_container: None,
        }
    }

    /// Set the condition.
    pub fn set_condition(&mut self, condition: bool) {
        if self.condition != condition {
            self.condition = condition;
            self.update();
        }
    }

    /// Update the view based on the current condition.
    fn update(&self) {
        if let (Some(template), Some(container)) = (&self.template, &self.view_container) {
            if self.condition {
                // Show the template
                let _ = container.append_child(template);
            } else {
                // Hide the template
                if template.parent_node().is_some() {
                    let _ = container.remove_child(template);
                }
            }
        }
    }
}

impl Default for IfDirective {
    fn default() -> Self {
        Self::new()
    }
}

/// The *for directive - renders a template for each item in a collection.
pub struct ForDirective<T> {
    items: Vec<T>,
    track_by: Option<Box<dyn Fn(&T) -> String>>,
    template: Option<web_sys::Element>,
    view_container: Option<web_sys::Element>,
    views: Vec<web_sys::Element>,
}

impl<T> ForDirective<T> {
    /// Create a new *for directive.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            track_by: None,
            template: None,
            view_container: None,
            views: Vec::new(),
        }
    }

    /// Set the items to iterate over.
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.update();
    }

    /// Set the track-by function.
    pub fn track_by<F>(&mut self, f: F)
    where
        F: Fn(&T) -> String + 'static,
    {
        self.track_by = Some(Box::new(f));
    }

    /// Update the views based on the current items.
    fn update(&mut self) {
        // In a real implementation, this would:
        // 1. Diff the old and new items using track_by
        // 2. Create, update, or remove views as needed
        // 3. Maintain view state for items that haven't changed
    }
}

impl<T> Default for ForDirective<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// The *switch directive - conditionally renders one of several templates.
pub struct SwitchDirective<T> {
    value: Option<T>,
    cases: Vec<(T, web_sys::Element)>,
    default_case: Option<web_sys::Element>,
    view_container: Option<web_sys::Element>,
    active_view: Option<web_sys::Element>,
}

impl<T: PartialEq> SwitchDirective<T> {
    /// Create a new *switch directive.
    pub fn new() -> Self {
        Self {
            value: None,
            cases: Vec::new(),
            default_case: None,
            view_container: None,
            active_view: None,
        }
    }

    /// Set the switch value.
    pub fn set_value(&mut self, value: T) {
        self.value = Some(value);
        self.update();
    }

    /// Add a case.
    pub fn add_case(&mut self, value: T, template: web_sys::Element) {
        self.cases.push((value, template));
    }

    /// Set the default case.
    pub fn set_default(&mut self, template: web_sys::Element) {
        self.default_case = Some(template);
    }

    /// Update the active view based on the current value.
    fn update(&mut self) {
        if let Some(ref value) = self.value {
            // Find matching case
            for (case_value, template) in &self.cases {
                if case_value == value {
                    self.activate_view(template.clone());
                    return;
                }
            }

            // Fall back to default
            if let Some(ref default) = self.default_case {
                self.activate_view(default.clone());
            }
        }
    }

    fn activate_view(&mut self, view: web_sys::Element) {
        // Deactivate current view
        if let Some(ref active) = self.active_view {
            if let Some(parent) = active.parent_node() {
                let _ = parent.remove_child(active);
            }
        }

        // Activate new view
        if let Some(ref container) = self.view_container {
            let _ = container.append_child(&view);
        }
        self.active_view = Some(view);
    }
}

impl<T: PartialEq> Default for SwitchDirective<T> {
    fn default() -> Self {
        Self::new()
    }
}

