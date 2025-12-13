//! validators! macro implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Result, Token};

struct ValidatorsInput {
    validators: Vec<Expr>,
}

impl Parse for ValidatorsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut validators = Vec::new();

        while !input.is_empty() {
            validators.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { validators })
    }
}

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let parsed: ValidatorsInput = syn::parse2(input)?;
    let validators = &parsed.validators;

    Ok(quote! {
        vec![#(Box::new(#validators)),*]
    })
}

