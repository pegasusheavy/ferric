//! Input and Output macro implementations.

use proc_macro2::TokenStream;
use quote::quote;

use crate::utils;

/// Implement the #[input] attribute macro.
pub fn input_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // For field attributes, just pass through with documentation
    // The component macro will process these annotations later

    let doc = if args.is_empty() {
        "Input property".to_string()
    } else {
        format!("Input property with args: {}", args)
    };
    let doc_lit = utils::lit_str(&doc);

    let expanded = quote! {
        #[doc = #doc_lit]
        #input
    };

    Ok(expanded)
}

/// Implement the #[output] attribute macro.
pub fn output_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // For field attributes, just pass through with documentation
    // The component macro will process these annotations later

    let doc = if args.is_empty() {
        "Output event".to_string()
    } else {
        format!("Output event with args: {}", args)
    };
    let doc_lit = utils::lit_str(&doc);

    let expanded = quote! {
        #[doc = #doc_lit]
        #input
    };

    Ok(expanded)
}
