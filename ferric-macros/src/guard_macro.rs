//! Route guard decorator implementation (Angular CanActivate, CanDeactivate, etc.).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct, Result};
use darling::FromMeta;

/// Arguments for the #[guard] decorator.
#[derive(Debug, FromMeta)]
struct GuardArgs {
    /// Guard type: "can_activate", "can_deactivate", "can_load", "can_activate_child"
    #[darling(rename = "type")]
    guard_type: String,

    /// Redirect path on failure
    #[darling(default)]
    redirect: Option<String>,
}

/// Implementation for #[guard] decorator.
pub fn guard_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let item: ItemStruct = parse2(input.clone())?;
    let name = &item.ident;

    // Default guard type is can_activate
    let guard_args = GuardArgs {
        guard_type: "can_activate".to_string(),
        redirect: None,
    };

    let guard_type = &guard_args.guard_type;
    let redirect = guard_args.redirect.as_ref()
        .map(|r| quote! { Some(#r.to_string()) })
        .unwrap_or_else(|| quote! { None });

    let trait_impl = match guard_type.as_str() {
        "can_activate" => quote! {
            impl ::ferric_core::router::CanActivate for #name {
                fn can_activate(
                    &self,
                    route: &::ferric_core::router::ActivatedRouteSnapshot,
                    state: &::ferric_core::router::RouterStateSnapshot,
                ) -> ::ferric_core::router::GuardResult {
                    self.check(route, state)
                }

                fn redirect_url(&self) -> Option<String> {
                    #redirect
                }
            }
        },
        "can_deactivate" => quote! {
            impl ::ferric_core::router::CanDeactivate for #name {
                fn can_deactivate(
                    &self,
                    component: &dyn std::any::Any,
                    current_route: &::ferric_core::router::ActivatedRouteSnapshot,
                    current_state: &::ferric_core::router::RouterStateSnapshot,
                    next_state: &::ferric_core::router::RouterStateSnapshot,
                ) -> ::ferric_core::router::GuardResult {
                    self.check_deactivate(component, current_route, current_state, next_state)
                }
            }
        },
        "can_load" => quote! {
            impl ::ferric_core::router::CanLoad for #name {
                fn can_load(
                    &self,
                    route: &::ferric_core::router::Route,
                ) -> ::ferric_core::router::GuardResult {
                    self.check_load(route)
                }
            }
        },
        "can_activate_child" => quote! {
            impl ::ferric_core::router::CanActivateChild for #name {
                fn can_activate_child(
                    &self,
                    child_route: &::ferric_core::router::ActivatedRouteSnapshot,
                    state: &::ferric_core::router::RouterStateSnapshot,
                ) -> ::ferric_core::router::GuardResult {
                    self.check_child(child_route, state)
                }
            }
        },
        _ => return Err(syn::Error::new_spanned(
            &guard_args.guard_type,
            format!("Invalid guard type: '{}'. Must be one of: can_activate, can_deactivate, can_load, can_activate_child", guard_type)
        )),
    };

    Ok(quote! {
        #item

        #trait_impl

        impl ::ferric_core::router::Guard for #name {
            fn guard_type(&self) -> &str {
                #guard_type
            }
        }
    })
}

