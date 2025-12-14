//! DSL for declarative form building.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Expr, Ident, Result, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

/// Form field specification in DSL.
struct FormField {
    name: Ident,
    _colon: Token![:],
    control: Expr,
}

impl Parse for FormField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _colon: input.parse()?,
            control: input.parse()?,
        })
    }
}

/// Form specification.
struct FormSpec {
    fields: Punctuated<FormField, Token![,]>,
}

impl Parse for FormSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            fields: input.parse_terminated(FormField::parse, Token![,])?,
        })
    }
}

/// Implementation for the `build_form!` macro.
pub fn build_form_impl(input: TokenStream) -> Result<TokenStream> {
    let spec: FormSpec = parse2(input)?;

    let field_names: Vec<_> = spec.fields.iter().map(|f| &f.name).collect();
    let field_controls: Vec<_> = spec.fields.iter().map(|f| &f.control).collect();

    Ok(quote! {
        {
            let mut group = ::ferric_forms::FormGroup::new();
            #(
                group.add_control(stringify!(#field_names), ::std::rc::Rc::new(#field_controls));
            )*
            group
        }
    })
}

/// Implementation for the `reactive_form!` macro with validation.
pub fn reactive_form_impl(input: TokenStream) -> Result<TokenStream> {
    let spec: FormSpec = parse2(input)?;

    let field_names: Vec<_> = spec.fields.iter().map(|f| &f.name).collect();
    let field_controls: Vec<_> = spec.fields.iter().map(|f| &f.control).collect();

    Ok(quote! {
        {
            use ::ferric_core::reactive::signal;
            use ::ferric_forms::FormGroup;

            let mut group = FormGroup::new();
            #(
                group.add_control(stringify!(#field_names), ::std::rc::Rc::new(#field_controls));
            )*

            // Make it reactive
            let form_signal = signal(group);
            form_signal
        }
    })
}

