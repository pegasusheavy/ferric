//! # Ferric Forms
//!
//! Reactive forms for the Ferric framework, inspired by Angular's Reactive Forms.
//!
//! ## Features
//!
//! - **FormControl** - Single input with value, validation, and state tracking
//! - **FormGroup** - Group of controls with collective validation
//! - **FormArray** - Dynamic list of controls
//! - **Validators** - Built-in and custom validation functions
//! - **Async Validators** - Server-side validation with debouncing
//! - **Cross-Field Validation** - Validators that span multiple controls
//! - **ControlValueAccessor** - Custom form control integration
//! - **Form Directives** - Template-driven form support
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_forms::prelude::*;
//!
//! // Create a form group
//! let form = form_group! {
//!     name: FormControl::new("", validators![required, min_length(2)]),
//!     email: FormControl::new("", validators![required, email]),
//!     age: FormControl::new(0, validators![required, min(18)]),
//! };
//!
//! // Add cross-field validation
//! form.set_cross_validators(vec![
//!     Box::new(password_match("password", "confirmPassword")),
//! ]);
//!
//! // Check validity
//! if form.valid() {
//!     let values = form.value();
//!     // Submit form...
//! }
//! ```

pub mod controls;
pub mod validators;
pub mod state;
pub mod binding;
pub mod accessor;
pub mod directives;

// Re-export macros
pub use ferric_forms_macros::{
    form_group,
    form_array,
    form_control,
    validators,
    Validate,
};

// Re-export main types
pub use controls::{
    FormControl,
    FormGroup,
    FormArray,
    AbstractControl,
    ControlContainer,
    // Typed group access
    TypedFormGroup,
    TypedFormWrapper,
    FormValueResult,
    FormPath,
};

pub use validators::{
    Validator,
    ValidatorFn,
    AsyncValidator,
    AsyncValidatorFn,
    ValidationErrors,
    ValidationResult,
    ValidationError,
    // Built-in validators
    required,
    required_true,
    min,
    max,
    min_length,
    max_length,
    pattern,
    email,
    // Cross-field validators
    CrossFieldValidatorTrait,
    CrossFieldValidator,
    MatchFieldsValidator,
    RequireOneOfValidator,
    AllOrNoneValidator,
    DateRangeValidator,
    NumericRangeValidator,
    ConditionalRequiredValidator,
    password_match,
    date_range,
    require_one_of,
    // Async validators
    DebouncedValidator,
    ConfiguredAsyncValidator,
    AsyncValidatorBuilder,
    UniqueValidator,
    ComposedAsyncValidator,
    async_custom,
    async_validator,
    unique_email,
    unique_username,
};

pub use state::{
    FormState,
    ControlStatus,
};

pub use binding::{
    FormBinding,
    InputBinding,
    UpdateOn,
};

pub use accessor::{
    ControlValueAccessor,
    ControlValueAccessorAny,
    DefaultValueAccessor,
    CheckboxValueAccessor,
    NumberValueAccessor,
    SelectValueAccessor,
    MultiSelectValueAccessor,
    RadioValueAccessor,
};

pub use directives::{
    FeModelDirective,
    FeFormGroupDirective,
    FeFormControlNameDirective,
    FeFormArrayNameDirective,
    FeFormControlDirective,
    FormDirectiveContainer,
    FormConfig,
    DirectiveAttributes,
};

/// Prelude module - import everything you need with `use ferric_forms::prelude::*`
pub mod prelude {
    pub use crate::controls::{
        FormControl,
        FormGroup,
        FormArray,
        AbstractControl,
        ControlContainer,
        TypedFormGroup,
        TypedFormWrapper,
        FormValueResult,
        FormPath,
    };

    pub use crate::validators::{
        Validator,
        ValidatorFn,
        AsyncValidator,
        ValidationErrors,
        ValidationResult,
        ValidationError,
        // Built-in validators
        required,
        required_true,
        min,
        max,
        min_length,
        max_length,
        pattern,
        email,
        // Cross-field validators
        CrossFieldValidatorTrait,
        CrossFieldValidator,
        MatchFieldsValidator,
        password_match,
        date_range,
        require_one_of,
        // Async validators
        DebouncedValidator,
        async_custom,
        async_validator,
        unique_email,
        unique_username,
    };

    pub use crate::state::{
        FormState,
        ControlStatus,
    };

    pub use crate::binding::{
        FormBinding,
        InputBinding,
        UpdateOn,
    };

    pub use crate::accessor::{
        ControlValueAccessor,
        DefaultValueAccessor,
        CheckboxValueAccessor,
        NumberValueAccessor,
        SelectValueAccessor,
    };

    pub use crate::directives::{
        FeModelDirective,
        FeFormGroupDirective,
        FeFormControlNameDirective,
        FeFormArrayNameDirective,
        FeFormControlDirective,
        FormDirectiveContainer,
        FormConfig,
    };

    pub use ferric_forms_macros::{
        form_group,
        form_array,
        form_control,
        validators,
        Validate,
    };
}
