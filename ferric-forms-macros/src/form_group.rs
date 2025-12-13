//! form_group! macro implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, Expr, Ident, Result, Token};

struct FormGroupField {
    name: Ident,
    value: Expr,
}

impl Parse for FormGroupField {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Expr = input.parse()?;
        Ok(Self { name, value })
    }
}

struct FormGroupInput {
    fields: Vec<FormGroupField>,
}

impl Parse for FormGroupInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _ = braced!(content in input);

        let mut fields = Vec::new();
        while !content.is_empty() {
            fields.push(content.parse()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(Self { fields })
    }
}

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let parsed: FormGroupInput = syn::parse2(input)?;

    let field_insertions = parsed.fields.iter().map(|field| {
        let name = &field.name;
        let name_str = name.to_string();
        let value = &field.value;

        quote! {
            controls.insert(
                #name_str.to_string(),
                ::std::rc::Rc::new(#value) as ::std::rc::Rc<dyn ::ferric_forms::AbstractControl>
            );
        }
    });

    Ok(quote! {
        {
            let mut controls = ::std::collections::HashMap::new();
            #(#field_insertions)*
            ::ferric_forms::FormGroup::with_controls(controls)
        }
    })
}

