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
use syn::{parse_macro_input, DeriveInput, Expr, ExprArray, ExprCall, Ident, Token};
use darling::{FromDeriveInput, FromField, FromMeta};

mod form_group;
mod form_control;
mod validators;
mod validate_derive;

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

