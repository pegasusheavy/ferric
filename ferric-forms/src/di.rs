//! Dependency Injection integration for Forms.

use crate::{
    FormControl, FormGroup, FormArray,
    validators::{Validator, AsyncValidator},
};
use ferric_core::di::{InjectionToken, MultiToken, Injector, Injectable};

/// Injection tokens for Forms configuration.
pub mod tokens {
    use super::*;

    /// Global form validators that apply to all forms.
    pub const FORM_GLOBAL_VALIDATORS: MultiToken<Box<dyn Validator<String>>> =
        MultiToken::with_id("FORM_GLOBAL_VALIDATORS", 3001);

    /// Global async validators.
    pub const FORM_GLOBAL_ASYNC_VALIDATORS: MultiToken<Box<dyn AsyncValidator<String>>> =
        MultiToken::with_id("FORM_GLOBAL_ASYNC_VALIDATORS", 3002);

    /// Default update strategy (on change, on blur, on submit).
    pub const FORM_DEFAULT_UPDATE_ON: InjectionToken<String> =
        InjectionToken::with_id("FORM_DEFAULT_UPDATE_ON", 3003);

    /// Enable form state persistence.
    pub const FORM_ENABLE_PERSISTENCE: InjectionToken<bool> =
        InjectionToken::with_id("FORM_ENABLE_PERSISTENCE", 3004);
}

/// Form builder service that uses DI for validators.
pub struct FormBuilder {
    injector: std::rc::Rc<Injector>,
}

impl FormBuilder {
    pub fn new(injector: std::rc::Rc<Injector>) -> Self {
        Self { injector }
    }

    pub fn control<T: Clone + 'static>(&self, initial_value: T) -> FormControl<T> {
        FormControl::new(initial_value)
    }

    pub fn group(&self) -> FormGroup {
        FormGroup::new()
    }

    pub fn array(&self) -> FormArray {
        FormArray::new()
    }
}

impl Injectable for FormBuilder {
    fn create(_injector: &Injector) -> Self {
        let injector_rc = std::rc::Rc::new(Injector::root());
        Self::new(injector_rc)
    }
}

/// Forms module for DI setup.
pub struct FormsModule;

impl FormsModule {
    pub fn provide(injector: &Injector) {
        injector.register_singleton::<FormBuilder>();
    }
}

