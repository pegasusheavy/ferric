//! Dependency Injection helper macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Type, Result};

/// Implementation for the `inject!` macro.
pub fn inject_impl(input: TokenStream) -> Result<TokenStream> {
    let ty: Type = parse2(input)?;

    Ok(quote! {
        {
            use ::ferric_core::di::Injector;
            let injector = Injector::current()
                .expect("No injector available in current context");
            injector.resolve::<#ty>()
                .expect(concat!("Failed to resolve dependency: ", stringify!(#ty)))
        }
    })
}

/// Implementation for the `provide!` macro.
pub fn provide_impl(input: TokenStream) -> Result<TokenStream> {
    let ty: Type = parse2(input)?;

    Ok(quote! {
        {
            use ::ferric_core::di::Injector;
            let injector = Injector::current()
                .expect("No injector available in current context");
            injector.register_singleton::<#ty>();
        }
    })
}

