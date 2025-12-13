//! ControlValueAccessor for custom form control integration.
//!
//! This module provides the interface for integrating custom UI components
//! with the forms system, similar to Angular's ControlValueAccessor.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_forms::prelude::*;
//!
//! struct ColorPicker {
//!     value: Signal<String>,
//!     on_change: Option<Box<dyn Fn(String)>>,
//!     on_touched: Option<Box<dyn Fn()>>,
//! }
//!
//! impl ControlValueAccessor for ColorPicker {
//!     type Value = String;
//!
//!     fn write_value(&mut self, value: String) {
//!         self.value.set(value);
//!     }
//!
//!     fn register_on_change(&mut self, fn_: Box<dyn Fn(String)>) {
//!         self.on_change = Some(fn_);
//!     }
//!
//!     fn register_on_touched(&mut self, fn_: Box<dyn Fn()>) {
//!         self.on_touched = Some(fn_);
//!     }
//!
//!     fn set_disabled_state(&mut self, disabled: bool) {
//!         // Update UI to show disabled state
//!     }
//! }
//! ```

use std::any::Any;

/// Trait for custom form control components.
///
/// Implement this trait to create custom form controls that can be
/// bound to `FormControl` instances.
pub trait ControlValueAccessor {
    /// The type of value this accessor handles.
    type Value: Clone;

    /// Write a new value to the accessor.
    ///
    /// Called when the form control's value changes programmatically.
    fn write_value(&mut self, value: Self::Value);

    /// Register a callback to be invoked when the value changes.
    ///
    /// The custom control should call this callback whenever
    /// the user changes the value.
    fn register_on_change(&mut self, callback: Box<dyn Fn(Self::Value)>);

    /// Register a callback to be invoked when the control is touched.
    ///
    /// The custom control should call this callback when the user
    /// interacts with it (e.g., on blur).
    fn register_on_touched(&mut self, callback: Box<dyn Fn()>);

    /// Set the disabled state of the control.
    fn set_disabled_state(&mut self, disabled: bool);

    /// Get the current value (optional, for convenience).
    fn read_value(&self) -> Option<Self::Value> {
        None
    }
}

/// Type-erased ControlValueAccessor for storage.
pub trait ControlValueAccessorAny {
    /// Write a value (as Any).
    fn write_value_any(&mut self, value: &dyn Any);

    /// Set disabled state.
    fn set_disabled_any(&mut self, disabled: bool);

    /// Get the accessor's type name for debugging.
    fn type_name(&self) -> &'static str;
}

/// Wrapper to convert typed accessor to type-erased.
pub struct AccessorWrapper<A>
where
    A: ControlValueAccessor,
    A::Value: 'static,
{
    accessor: A,
}

impl<A> AccessorWrapper<A>
where
    A: ControlValueAccessor,
    A::Value: 'static,
{
    /// Create a new wrapper.
    pub fn new(accessor: A) -> Self {
        Self { accessor }
    }

    /// Get mutable access to the inner accessor.
    pub fn inner_mut(&mut self) -> &mut A {
        &mut self.accessor
    }
}

impl<A> ControlValueAccessorAny for AccessorWrapper<A>
where
    A: ControlValueAccessor,
    A::Value: 'static + Clone,
{
    fn write_value_any(&mut self, value: &dyn Any) {
        if let Some(typed_value) = value.downcast_ref::<A::Value>() {
            self.accessor.write_value(typed_value.clone());
        }
    }

    fn set_disabled_any(&mut self, disabled: bool) {
        self.accessor.set_disabled_state(disabled);
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<A>()
    }
}

// ============================================================================
// Built-in Accessors for Standard Input Types
// ============================================================================

/// Default accessor for text inputs.
pub struct DefaultValueAccessor {
    on_change: Option<Box<dyn Fn(String)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl DefaultValueAccessor {
    /// Create a new default value accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify that the value changed.
    pub fn value_changed(&self, value: String) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }

    /// Notify that the control was touched.
    pub fn touched(&self) {
        if let Some(ref callback) = self.on_touched {
            callback();
        }
    }

    /// Check if disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Default for DefaultValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for DefaultValueAccessor {
    type Value = String;

    fn write_value(&mut self, _value: String) {
        // The actual DOM update would happen through bindings
    }

    fn register_on_change(&mut self, callback: Box<dyn Fn(String)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for checkbox inputs.
pub struct CheckboxValueAccessor {
    on_change: Option<Box<dyn Fn(bool)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl CheckboxValueAccessor {
    /// Create a new checkbox accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify value change.
    pub fn value_changed(&self, checked: bool) {
        if let Some(ref callback) = self.on_change {
            callback(checked);
        }
    }
}

impl Default for CheckboxValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for CheckboxValueAccessor {
    type Value = bool;

    fn write_value(&mut self, _value: bool) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(bool)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for number inputs.
pub struct NumberValueAccessor {
    on_change: Option<Box<dyn Fn(f64)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl NumberValueAccessor {
    /// Create a new number accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify value change.
    pub fn value_changed(&self, value: f64) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }
}

impl Default for NumberValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for NumberValueAccessor {
    type Value = f64;

    fn write_value(&mut self, _value: f64) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(f64)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for select inputs.
pub struct SelectValueAccessor {
    on_change: Option<Box<dyn Fn(String)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
    multiple: bool,
}

impl SelectValueAccessor {
    /// Create a new select accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
            multiple: false,
        }
    }

    /// Set multiple selection mode.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Notify value change.
    pub fn value_changed(&self, value: String) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }
}

impl Default for SelectValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for SelectValueAccessor {
    type Value = String;

    fn write_value(&mut self, _value: String) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(String)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for multi-select inputs.
pub struct MultiSelectValueAccessor {
    on_change: Option<Box<dyn Fn(Vec<String>)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl MultiSelectValueAccessor {
    /// Create a new multi-select accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify value change.
    pub fn value_changed(&self, values: Vec<String>) {
        if let Some(ref callback) = self.on_change {
            callback(values);
        }
    }
}

impl Default for MultiSelectValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for MultiSelectValueAccessor {
    type Value = Vec<String>;

    fn write_value(&mut self, _value: Vec<String>) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(Vec<String>)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for radio button groups.
pub struct RadioValueAccessor {
    name: String,
    on_change: Option<Box<dyn Fn(String)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl RadioValueAccessor {
    /// Create a new radio accessor.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Get the radio group name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Notify value change.
    pub fn value_changed(&self, value: String) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }
}

impl ControlValueAccessor for RadioValueAccessor {
    type Value = String;

    fn write_value(&mut self, _value: String) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(String)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_value_accessor() {
        let mut accessor = DefaultValueAccessor::new();
        let called = std::rc::Rc::new(std::cell::RefCell::new(false));

        let called_clone = called.clone();
        accessor.register_on_change(Box::new(move |_| {
            *called_clone.borrow_mut() = true;
        }));

        accessor.value_changed("test".to_string());
        assert!(*called.borrow());
    }

    #[test]
    fn test_checkbox_accessor() {
        let mut accessor = CheckboxValueAccessor::new();
        let value = std::rc::Rc::new(std::cell::RefCell::new(false));

        let value_clone = value.clone();
        accessor.register_on_change(Box::new(move |v| {
            *value_clone.borrow_mut() = v;
        }));

        accessor.value_changed(true);
        assert!(*value.borrow());
    }
}


//!
//! This module provides the interface for integrating custom UI components
//! with the forms system, similar to Angular's ControlValueAccessor.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_forms::prelude::*;
//!
//! struct ColorPicker {
//!     value: Signal<String>,
//!     on_change: Option<Box<dyn Fn(String)>>,
//!     on_touched: Option<Box<dyn Fn()>>,
//! }
//!
//! impl ControlValueAccessor for ColorPicker {
//!     type Value = String;
//!
//!     fn write_value(&mut self, value: String) {
//!         self.value.set(value);
//!     }
//!
//!     fn register_on_change(&mut self, fn_: Box<dyn Fn(String)>) {
//!         self.on_change = Some(fn_);
//!     }
//!
//!     fn register_on_touched(&mut self, fn_: Box<dyn Fn()>) {
//!         self.on_touched = Some(fn_);
//!     }
//!
//!     fn set_disabled_state(&mut self, disabled: bool) {
//!         // Update UI to show disabled state
//!     }
//! }
//! ```

use std::any::Any;

/// Trait for custom form control components.
///
/// Implement this trait to create custom form controls that can be
/// bound to `FormControl` instances.
pub trait ControlValueAccessor {
    /// The type of value this accessor handles.
    type Value: Clone;

    /// Write a new value to the accessor.
    ///
    /// Called when the form control's value changes programmatically.
    fn write_value(&mut self, value: Self::Value);

    /// Register a callback to be invoked when the value changes.
    ///
    /// The custom control should call this callback whenever
    /// the user changes the value.
    fn register_on_change(&mut self, callback: Box<dyn Fn(Self::Value)>);

    /// Register a callback to be invoked when the control is touched.
    ///
    /// The custom control should call this callback when the user
    /// interacts with it (e.g., on blur).
    fn register_on_touched(&mut self, callback: Box<dyn Fn()>);

    /// Set the disabled state of the control.
    fn set_disabled_state(&mut self, disabled: bool);

    /// Get the current value (optional, for convenience).
    fn read_value(&self) -> Option<Self::Value> {
        None
    }
}

/// Type-erased ControlValueAccessor for storage.
pub trait ControlValueAccessorAny {
    /// Write a value (as Any).
    fn write_value_any(&mut self, value: &dyn Any);

    /// Set disabled state.
    fn set_disabled_any(&mut self, disabled: bool);

    /// Get the accessor's type name for debugging.
    fn type_name(&self) -> &'static str;
}

/// Wrapper to convert typed accessor to type-erased.
pub struct AccessorWrapper<A>
where
    A: ControlValueAccessor,
    A::Value: 'static,
{
    accessor: A,
}

impl<A> AccessorWrapper<A>
where
    A: ControlValueAccessor,
    A::Value: 'static,
{
    /// Create a new wrapper.
    pub fn new(accessor: A) -> Self {
        Self { accessor }
    }

    /// Get mutable access to the inner accessor.
    pub fn inner_mut(&mut self) -> &mut A {
        &mut self.accessor
    }
}

impl<A> ControlValueAccessorAny for AccessorWrapper<A>
where
    A: ControlValueAccessor,
    A::Value: 'static + Clone,
{
    fn write_value_any(&mut self, value: &dyn Any) {
        if let Some(typed_value) = value.downcast_ref::<A::Value>() {
            self.accessor.write_value(typed_value.clone());
        }
    }

    fn set_disabled_any(&mut self, disabled: bool) {
        self.accessor.set_disabled_state(disabled);
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<A>()
    }
}

// ============================================================================
// Built-in Accessors for Standard Input Types
// ============================================================================

/// Default accessor for text inputs.
pub struct DefaultValueAccessor {
    on_change: Option<Box<dyn Fn(String)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl DefaultValueAccessor {
    /// Create a new default value accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify that the value changed.
    pub fn value_changed(&self, value: String) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }

    /// Notify that the control was touched.
    pub fn touched(&self) {
        if let Some(ref callback) = self.on_touched {
            callback();
        }
    }

    /// Check if disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Default for DefaultValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for DefaultValueAccessor {
    type Value = String;

    fn write_value(&mut self, _value: String) {
        // The actual DOM update would happen through bindings
    }

    fn register_on_change(&mut self, callback: Box<dyn Fn(String)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for checkbox inputs.
pub struct CheckboxValueAccessor {
    on_change: Option<Box<dyn Fn(bool)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl CheckboxValueAccessor {
    /// Create a new checkbox accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify value change.
    pub fn value_changed(&self, checked: bool) {
        if let Some(ref callback) = self.on_change {
            callback(checked);
        }
    }
}

impl Default for CheckboxValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for CheckboxValueAccessor {
    type Value = bool;

    fn write_value(&mut self, _value: bool) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(bool)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for number inputs.
pub struct NumberValueAccessor {
    on_change: Option<Box<dyn Fn(f64)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl NumberValueAccessor {
    /// Create a new number accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify value change.
    pub fn value_changed(&self, value: f64) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }
}

impl Default for NumberValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for NumberValueAccessor {
    type Value = f64;

    fn write_value(&mut self, _value: f64) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(f64)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for select inputs.
pub struct SelectValueAccessor {
    on_change: Option<Box<dyn Fn(String)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
    multiple: bool,
}

impl SelectValueAccessor {
    /// Create a new select accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
            multiple: false,
        }
    }

    /// Set multiple selection mode.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Notify value change.
    pub fn value_changed(&self, value: String) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }
}

impl Default for SelectValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for SelectValueAccessor {
    type Value = String;

    fn write_value(&mut self, _value: String) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(String)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for multi-select inputs.
pub struct MultiSelectValueAccessor {
    on_change: Option<Box<dyn Fn(Vec<String>)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl MultiSelectValueAccessor {
    /// Create a new multi-select accessor.
    pub fn new() -> Self {
        Self {
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Notify value change.
    pub fn value_changed(&self, values: Vec<String>) {
        if let Some(ref callback) = self.on_change {
            callback(values);
        }
    }
}

impl Default for MultiSelectValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlValueAccessor for MultiSelectValueAccessor {
    type Value = Vec<String>;

    fn write_value(&mut self, _value: Vec<String>) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(Vec<String>)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// Accessor for radio button groups.
pub struct RadioValueAccessor {
    name: String,
    on_change: Option<Box<dyn Fn(String)>>,
    on_touched: Option<Box<dyn Fn()>>,
    disabled: bool,
}

impl RadioValueAccessor {
    /// Create a new radio accessor.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            on_change: None,
            on_touched: None,
            disabled: false,
        }
    }

    /// Get the radio group name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Notify value change.
    pub fn value_changed(&self, value: String) {
        if let Some(ref callback) = self.on_change {
            callback(value);
        }
    }
}

impl ControlValueAccessor for RadioValueAccessor {
    type Value = String;

    fn write_value(&mut self, _value: String) {}

    fn register_on_change(&mut self, callback: Box<dyn Fn(String)>) {
        self.on_change = Some(callback);
    }

    fn register_on_touched(&mut self, callback: Box<dyn Fn()>) {
        self.on_touched = Some(callback);
    }

    fn set_disabled_state(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_value_accessor() {
        let mut accessor = DefaultValueAccessor::new();
        let called = std::rc::Rc::new(std::cell::RefCell::new(false));

        let called_clone = called.clone();
        accessor.register_on_change(Box::new(move |_| {
            *called_clone.borrow_mut() = true;
        }));

        accessor.value_changed("test".to_string());
        assert!(*called.borrow());
    }

    #[test]
    fn test_checkbox_accessor() {
        let mut accessor = CheckboxValueAccessor::new();
        let value = std::rc::Rc::new(std::cell::RefCell::new(false));

        let value_clone = value.clone();
        accessor.register_on_change(Box::new(move |v| {
            *value_clone.borrow_mut() = v;
        }));

        accessor.value_changed(true);
        assert!(*value.borrow());
    }
}

