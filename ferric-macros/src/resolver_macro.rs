//! Route resolver decorator implementation (Angular Resolve equivalent).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct, Result};
use darling::FromMeta;

/// Arguments for the #[resolver] decorator.
#[derive(Debug, FromMeta)]
struct ResolverArgs {
    /// The type this resolver produces
    #[darling(default)]
    resolves: Option<String>,
}

/// Implementation for #[resolver] decorator.
pub fn resolver_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let item: ItemStruct = parse2(input.clone())?;
    let name = &item.ident;

    let resolver_args = ResolverArgs { resolves: None };

    let resolves_doc = resolver_args.resolves.as_ref()
        .map(|r| format!("Resolves: {}", r))
        .unwrap_or_else(|| "Generic resolver".to_string());

    Ok(quote! {
        #[doc = #resolves_doc]
        #item

        impl ::ferric_core::router::Resolve for #name {
            fn resolve(
                &self,
                route: &::ferric_core::router::ActivatedRouteSnapshot,
            ) -> ::ferric_core::router::ResolverResult {
                self.load_data(route)
            }
        }

        impl ::ferric_core::router::Resolver for #name {
            fn resolver_name(&self) -> &str {
                stringify!(#name)
            }
        }
    })
}

