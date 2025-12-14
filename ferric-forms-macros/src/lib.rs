//! Procedural macros for Ferric Forms.
//!
//! Provides declarative macros for building reactive forms:
//!
//! - `#[derive(Validate)]` - Derive validation for structs
//! - `form_group!` - Create form groups declaratively
//! - `form_array!` - Create form arrays
//! - `form_control!` - Create form controls with validators
//! - `validators!` - Compose validators

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ExprArray};
// darling is used in submodules

mod form_group;
mod form_control;
mod validators;
mod validate_derive;

// New utility macros
mod form_builder;
mod validator_macros;
mod form_dsl;

/// Derive macro for adding validation to structs.
///
/// # Example
///
/// ```ignore
/// #[derive(Validate)]
/// struct LoginForm {
///     #[validate(required, email)]
///     email: String,
///
///     #[validate(required, min_length = 8)]
///     password: String,
/// }
/// ```
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    validate_derive::expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a FormGroup declaratively.
///
/// # Example
///
/// ```ignore
/// let form = form_group! {
///     name: FormControl::text(""),
///     email: FormControl::text("").with_validators(vec![Box::new(email())]),
///     age: FormControl::number(0),
///     address: form_group! {
///         street: FormControl::text(""),
///         city: FormControl::text(""),
///         zip: FormControl::text(""),
///     },
/// };
/// ```
#[proc_macro]
pub fn form_group(input: TokenStream) -> TokenStream {
    form_group::expand(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a FormArray declaratively.
///
/// # Example
///
/// ```ignore
/// let phones = form_array![
///     FormControl::text("555-1234"),
///     FormControl::text("555-5678"),
/// ];
/// ```
#[proc_macro]
pub fn form_array(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExprArray);

    let items = input.elems.iter();

    let expanded = quote! {
        {
            let mut array = ::ferric_forms::FormArray::new();
            #(
                array.push(::std::rc::Rc::new(#items));
            )*
            array
        }
    };

    expanded.into()
}

/// Create a FormControl with validators.
///
/// # Example
///
/// ```ignore
/// let email = form_control!(String, "", [required(), email()]);
/// let age = form_control!(i32, 0, [required(), min(18), max(120)]);
/// ```
#[proc_macro]
pub fn form_control(input: TokenStream) -> TokenStream {
    form_control::expand(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Compose multiple validators.
///
/// # Example
///
/// ```ignore
/// let validators = validators![required(), min_length(2), max_length(50)];
/// ```
#[proc_macro]
pub fn validators(input: TokenStream) -> TokenStream {
    validators::expand(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Type-Safe Form Macros
// ============================================================================

/// Derive a typed form builder for a struct.
///
/// # Example
///
/// ```ignore
/// #[derive(TypedForm)]
/// struct User {
///     name: String,
///     email: String,
///     age: u32,
/// }
///
/// // Generates UserForm with typed controls
/// let form = UserForm::from_value(user);
/// ```
#[proc_macro_derive(TypedForm)]
pub fn derive_typed_form(input: TokenStream) -> TokenStream {
    form_builder::typed_form_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Build a form declaratively with DSL syntax.
///
/// # Example
///
/// ```ignore
/// let form = build_form! {
///     username: FormControl::text(""),
///     email: FormControl::text("").with_validators(validators![required(), email()]),
///     password: FormControl::password("").with_validators(validators![required(), min_length(8)]),
/// };
/// ```
#[proc_macro]
pub fn build_form(input: TokenStream) -> TokenStream {
    form_dsl::build_form_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a reactive form with signal integration.
///
/// # Example
///
/// ```ignore
/// let form = reactive_form! {
///     name: FormControl::text(""),
///     age: FormControl::number(0),
/// };
///
/// // form is a Signal<FormGroup>
/// effect!(|| {
///     if form.get().is_valid() {
///         println!("Form is valid!");
///     }
/// });
/// ```
#[proc_macro]
pub fn reactive_form(input: TokenStream) -> TokenStream {
    form_dsl::reactive_form_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

// ============================================================================
// Validator Composition Macros
// ============================================================================

/// Compose multiple validators into a vector.
///
/// # Example
///
/// ```ignore
/// let validators = compose_validators![
///     required(),
///     min_length(3),
///     max_length(50),
///     pattern(r"^[a-zA-Z]+$"),
/// ];
/// ```
#[proc_macro]
pub fn compose_validators(input: TokenStream) -> TokenStream {
    validator_macros::compose_validators_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a conditional required validator.
///
/// # Example
///
/// ```ignore
/// let validator = required_if!(age.get() >= 18);
/// ```
#[proc_macro]
pub fn required_if(input: TokenStream) -> TokenStream {
    validator_macros::required_if_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create a cross-field validator to match another field.
///
/// # Example
///
/// ```ignore
/// let validator = match_field!("password");
/// ```
#[proc_macro]
pub fn match_field(input: TokenStream) -> TokenStream {
    validator_macros::match_field_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Create an async validator.
///
/// # Example
///
/// ```ignore
/// let validator = async_validator!(|value: String| async move {
///     check_username_availability(value).await
/// });
/// ```
#[proc_macro]
pub fn async_validator(input: TokenStream) -> TokenStream {
    validator_macros::async_validator_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

