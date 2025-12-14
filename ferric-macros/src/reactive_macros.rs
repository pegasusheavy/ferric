//! Reactive programming helper macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Expr, ExprClosure, Result};

/// Implementation for the `computed!` macro.
pub fn computed_impl(input: TokenStream) -> Result<TokenStream> {
    let closure: ExprClosure = parse2(input)?;

    Ok(quote! {
        ::ferric_core::reactive::computed(#closure)
    })
}

/// Implementation for the `effect!` macro.
pub fn effect_impl(input: TokenStream) -> Result<TokenStream> {
    let closure: ExprClosure = parse2(input)?;

    Ok(quote! {
        ::ferric_core::reactive::effect(#closure)
    })
}

/// Implementation for the `signal!` macro with initial value.
pub fn signal_impl(input: TokenStream) -> Result<TokenStream> {
    let expr: Expr = parse2(input)?;

    Ok(quote! {
        ::ferric_core::reactive::signal(#expr)
    })
}

/// Implementation for the `batch!` macro.
pub fn batch_impl(input: TokenStream) -> Result<TokenStream> {
    let closure: ExprClosure = parse2(input)?;

    Ok(quote! {
        ::ferric_core::reactive::batch(#closure)
    })
}

/// Implementation for the `memo!` macro.
pub fn memo_impl(input: TokenStream) -> Result<TokenStream> {
    let closure: ExprClosure = parse2(input)?;

    Ok(quote! {
        {
            use ::std::cell::RefCell;
            use ::std::rc::Rc;

            thread_local! {
                static MEMO: RefCell<Option<_>> = RefCell::new(None);
            }

            MEMO.with(|cell| {
                let mut cache = cell.borrow_mut();
                if cache.is_none() {
                    *cache = Some((#closure)());
                }
                cache.as_ref().unwrap().clone()
            })
        }
    })
}

/// Implementation for the `watch!` macro.
pub fn watch_impl(input: TokenStream) -> Result<TokenStream> {
    let closure: ExprClosure = parse2(input)?;

    Ok(quote! {
        ::ferric_core::reactive::effect(#closure)
    })
}

/// Implementation for the `lazy!` macro.
pub fn lazy_impl(input: TokenStream) -> Result<TokenStream> {
    let closure: ExprClosure = parse2(input)?;

    Ok(quote! {
        {
            use ::std::sync::OnceLock;
            static LAZY: OnceLock<_> = OnceLock::new();
            LAZY.get_or_init(|| (#closure)())
        }
    })
}

