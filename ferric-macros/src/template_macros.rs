//! Template and HTML helper macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, LitStr, Result};

/// Implementation for the `html!` macro.
pub fn html_impl(input: TokenStream) -> Result<TokenStream> {
    let lit: LitStr = parse2(input)?;
    let html = lit.value();

    Ok(quote! {
        ::ferric_core::template::parse_template(#html)
            .expect("Invalid HTML template")
    })
}

/// Implementation for the `css!` macro.
pub fn css_impl(input: TokenStream) -> Result<TokenStream> {
    let lit: LitStr = parse2(input)?;
    let css = lit.value();

    Ok(quote! {
        {
            // In production, this could minify or process CSS
            #css.to_string()
        }
    })
}

/// Implementation for the `selector!` macro.
pub fn selector_impl(input: TokenStream) -> Result<TokenStream> {
    let lit: LitStr = parse2(input)?;
    let selector = lit.value();

    // Validate selector format
    if !selector.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(syn::Error::new_spanned(
            lit,
            "Selector must only contain alphanumeric characters, hyphens, and underscores",
        ));
    }

    Ok(quote! {
        #selector
    })
}

