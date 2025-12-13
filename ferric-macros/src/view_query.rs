//! View query macro implementations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

use crate::utils;

/// Implement the #[view_child] attribute macro.
pub fn view_child_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse selector from first argument
    let selector = if let Ok(lit) = syn::parse2::<LitStr>(args.clone()) {
        lit.value()
    } else {
        let args_str = args.to_string();
        args_str.trim_matches('"').to_string()
    };

    let selector_lit = utils::lit_str(&selector);

    // For field attributes, we just annotate and pass through
    // The component macro will process these later
    let expanded = quote! {
        #[doc = concat!("ViewChild query: ", #selector_lit)]
        #input
    };

    Ok(expanded)
}

/// Implement the #[view_children] attribute macro.
pub fn view_children_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse selector
    let selector = if let Ok(lit) = syn::parse2::<LitStr>(args.clone()) {
        lit.value()
    } else {
        let args_str = args.to_string();
        args_str.trim_matches('"').to_string()
    };

    let selector_lit = utils::lit_str(&selector);

    let expanded = quote! {
        #[doc = concat!("ViewChildren query: ", #selector_lit)]
        #input
    };

    Ok(expanded)
}

/// Implement the #[content_child] attribute macro.
pub fn content_child_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse selector
    let selector = if let Ok(lit) = syn::parse2::<LitStr>(args.clone()) {
        lit.value()
    } else {
        let args_str = args.to_string();
        args_str.trim_matches('"').to_string()
    };

    let selector_lit = utils::lit_str(&selector);

    let expanded = quote! {
        #[doc = concat!("ContentChild query: ", #selector_lit)]
        #input
    };

    Ok(expanded)
}

/// Implement the #[content_children] attribute macro.
pub fn content_children_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse selector
    let selector = if let Ok(lit) = syn::parse2::<LitStr>(args.clone()) {
        lit.value()
    } else {
        let args_str = args.to_string();
        args_str.trim_matches('"').to_string()
    };

    let selector_lit = utils::lit_str(&selector);

    let expanded = quote! {
        #[doc = concat!("ContentChildren query: ", #selector_lit)]
        #input
    };

    Ok(expanded)
}
