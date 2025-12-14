//! Module decorator implementation (Angular @NgModule equivalent).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct, Result};
use darling::FromMeta;

/// Arguments for the #[module] decorator.
#[derive(Debug, FromMeta)]
struct ModuleArgs {
    /// Imported modules
    #[darling(default)]
    imports: Option<syn::Expr>,

    /// Module providers (services)
    #[darling(default)]
    providers: Option<syn::Expr>,

    /// Module controllers/components
    #[darling(default)]
    declarations: Option<syn::Expr>,

    /// Exported components/directives
    #[darling(default)]
    exports: Option<syn::Expr>,

    /// Bootstrap components
    #[darling(default)]
    bootstrap: Option<syn::Expr>,
}

/// Implementation for #[module] decorator.
pub fn module_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let item: ItemStruct = parse2(input.clone())?;
    let name = &item.ident;

    // For now, just implement the trait without parsing complex args
    // Users can implement the module trait manually or we can enhance this later
    let module_args = ModuleArgs {
        imports: None,
        providers: None,
        declarations: None,
        exports: None,
        bootstrap: None,
    };

    let imports = module_args.imports.as_ref().map(|i| quote! { #i }).unwrap_or_else(|| quote! { vec![] });
    let providers = module_args.providers.as_ref().map(|p| quote! { #p }).unwrap_or_else(|| quote! { vec![] });
    let declarations = module_args.declarations.as_ref().map(|d| quote! { #d }).unwrap_or_else(|| quote! { vec![] });
    let exports = module_args.exports.as_ref().map(|e| quote! { #e }).unwrap_or_else(|| quote! { vec![] });
    let bootstrap = module_args.bootstrap.as_ref().map(|b| quote! { Some(#b) }).unwrap_or_else(|| quote! { None });

    Ok(quote! {
        #item

        impl ::ferric_core::module::Module for #name {
            fn imports(&self) -> Vec<Box<dyn ::ferric_core::module::Module>> {
                #imports
            }

            fn providers(&self) -> Vec<Box<dyn ::ferric_core::provider::Provider>> {
                #providers
            }

            fn declarations(&self) -> Vec<Box<dyn ::ferric_core::component::Component>> {
                #declarations
            }

            fn exports(&self) -> Vec<String> {
                #exports
            }

            fn bootstrap(&self) -> Option<String> {
                #bootstrap
            }
        }
    })
}

