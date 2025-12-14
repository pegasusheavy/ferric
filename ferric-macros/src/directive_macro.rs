//! Directive decorator implementation (Angular @Directive equivalent).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct, Result};
use darling::FromMeta;

/// Arguments for the #[directive] decorator.
#[derive(Debug, FromMeta)]
struct DirectiveArgs {
    /// Directive selector
    selector: String,

    /// Standalone directive (doesn't need NgModule)
    #[darling(default)]
    standalone: bool,

    /// Host bindings
    #[darling(default)]
    #[allow(dead_code)]
    host: Option<String>,

    /// Exported as (for template references)
    #[darling(default)]
    export_as: Option<String>,
}

/// Implementation for #[directive] decorator.
pub fn directive_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let item: ItemStruct = parse2(input.clone())?;
    let name = &item.ident;

    // Default directive args - selector is the struct name in kebab-case
    let selector = name.to_string().to_lowercase();
    let directive_args = DirectiveArgs {
        selector,
        standalone: false,
        host: None,
        export_as: None,
    };

    let selector = &directive_args.selector;
    let standalone = directive_args.standalone;
    let export_as = directive_args.export_as.as_ref()
        .map(|e| quote! { Some(#e.to_string()) })
        .unwrap_or_else(|| quote! { None });

    Ok(quote! {
        #item

        impl ::ferric_core::directive::Directive for #name {
            fn selector(&self) -> &str {
                #selector
            }

            fn is_standalone(&self) -> bool {
                #standalone
            }

            fn export_as(&self) -> Option<String> {
                #export_as
            }

            fn on_init(&self) {
                // Default implementation
            }

            fn on_destroy(&self) {
                // Default implementation
            }
        }

        impl ::ferric_core::directive::DirectiveMetadata for #name {
            fn metadata() -> ::ferric_core::directive::DirectiveMeta {
                ::ferric_core::directive::DirectiveMeta {
                    selector: #selector.to_string(),
                    standalone: #standalone,
                    export_as: #export_as,
                }
            }
        }
    })
}

