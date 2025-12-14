//! Testing helper macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn, Result};

/// Implementation for the `component_test!` macro.
pub fn component_test_impl(input: TokenStream) -> Result<TokenStream> {
    let func: ItemFn = parse2(input)?;
    let func_name = &func.sig.ident;
    let func_block = &func.block;

    Ok(quote! {
        #[test]
        fn #func_name() {
            use ::ferric_core::testing::TestBed;

            let test_bed = TestBed::configure().build();

            #func_block
        }
    })
}

/// Implementation for the `async_test!` macro.
pub fn async_test_impl(input: TokenStream) -> Result<TokenStream> {
    let func: ItemFn = parse2(input)?;
    let func_name = &func.sig.ident;
    let func_block = &func.block;

    Ok(quote! {
        #[tokio::test]
        async fn #func_name() {
            #func_block
        }
    })
}

