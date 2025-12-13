//! form_control! macro implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, Expr, Result, Token, Type};

struct FormControlInput {
    ty: Type,
    initial_value: Expr,
    validators: Vec<Expr>,
}

impl Parse for FormControlInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let initial_value: Expr = input.parse()?;

        let validators = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let content;
            bracketed!(content in input);

            let mut validators = Vec::new();
            while !content.is_empty() {
                validators.push(content.parse()?);
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
            validators
        } else {
            Vec::new()
        };

        Ok(Self {
            ty,
            initial_value,
            validators,
        })
    }
}

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let parsed: FormControlInput = syn::parse2(input)?;

    let ty = &parsed.ty;
    let initial_value = &parsed.initial_value;
    let validators = &parsed.validators;

    if validators.is_empty() {
        Ok(quote! {
            ::ferric_forms::FormControl::<#ty>::new(#initial_value)
        })
    } else {
        Ok(quote! {
            ::ferric_forms::FormControl::<#ty>::with_validators(
                #initial_value,
                vec![#(Box::new(#validators) as Box<dyn ::ferric_forms::Validator<#ty>>),*]
            )
        })
    }
}

