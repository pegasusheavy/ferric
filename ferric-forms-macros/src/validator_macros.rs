//! Validator composition macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Expr, Result};
use syn::punctuated::Punctuated;
use syn::Token;

/// Implementation for the `compose_validators!` macro.
pub fn compose_validators_impl(input: TokenStream) -> Result<TokenStream> {
    use syn::parse::Parser;

    let validators = Punctuated::<Expr, Token![,]>::parse_terminated
        .parse2(input)?;

    let validator_calls = validators.iter();

    Ok(quote! {
        vec![
            #(Box::new(#validator_calls) as Box<dyn ::ferric_forms::Validator>),*
        ]
    })
}

/// Implementation for the `required_if!` macro.
pub fn required_if_impl(input: TokenStream) -> Result<TokenStream> {
    let condition: Expr = parse2(input)?;

    Ok(quote! {
        ::ferric_forms::validators::ConditionalValidator::new(
            Box::new(::ferric_forms::validators::RequiredValidator),
            Box::new(|| #condition)
        )
    })
}

/// Implementation for the `match_field!` macro.
pub fn match_field_impl(input: TokenStream) -> Result<TokenStream> {
    let field_name: Expr = parse2(input)?;

    Ok(quote! {
        ::ferric_forms::validators::CrossFieldValidator::match_fields(
            #field_name,
            "Value must match"
        )
    })
}

/// Implementation for the `async_validator!` macro.
pub fn async_validator_impl(input: TokenStream) -> Result<TokenStream> {
    let closure: Expr = parse2(input)?;

    Ok(quote! {
        ::ferric_forms::validators::AsyncValidator::new(#closure)
    })
}

