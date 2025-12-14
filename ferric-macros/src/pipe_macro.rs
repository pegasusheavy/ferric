//! Pipe decorator implementation (Angular @Pipe equivalent).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct, Result};
use darling::FromMeta;

/// Arguments for the #[pipe] decorator.
#[derive(Debug, FromMeta)]
struct PipeArgs {
    /// Pipe name (used in templates)
    name: String,

    /// Pure pipe (default: true)
    #[darling(default = "default_pure")]
    pure: bool,

    /// Standalone pipe
    #[darling(default)]
    standalone: bool,
}

fn default_pure() -> bool {
    true
}

/// Implementation for #[pipe] decorator.
pub fn pipe_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let item: ItemStruct = parse2(input.clone())?;
    let name = &item.ident;

    // Default pipe name is the struct name in lowercase
    let pipe_name = name.to_string().to_lowercase();
    let pipe_args = PipeArgs {
        name: pipe_name,
        pure: true,
        standalone: false,
    };

    let pipe_name = &pipe_args.name;
    let pure = pipe_args.pure;
    let standalone = pipe_args.standalone;

    Ok(quote! {
        #item

        impl ::ferric_core::pipe::Pipe for #name {
            fn name(&self) -> &str {
                #pipe_name
            }

            fn is_pure(&self) -> bool {
                #pure
            }

            fn is_standalone(&self) -> bool {
                #standalone
            }
        }

        impl ::ferric_core::pipe::PipeMetadata for #name {
            fn metadata() -> ::ferric_core::pipe::PipeMeta {
                ::ferric_core::pipe::PipeMeta {
                    name: #pipe_name.to_string(),
                    pure: #pure,
                    standalone: #standalone,
                }
            }
        }
    })
}

