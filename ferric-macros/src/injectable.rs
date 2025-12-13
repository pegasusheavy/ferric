//! Injectable macro implementation.

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, DeriveInput, ItemStruct, Fields, Ident, Type};

/// Arguments for the #[injectable] attribute.
#[derive(Debug, Default, FromMeta)]
pub struct InjectableArgs {
    /// Where to provide the service.
    #[darling(default)]
    provided_in: Option<String>,
}

/// Implement the #[injectable] attribute macro.
pub fn injectable_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse arguments
    let attr_args = if args.is_empty() {
        InjectableArgs::default()
    } else {
        let nested = darling::ast::NestedMeta::parse_meta_list(args)?;
        InjectableArgs::from_list(&nested)?
    };

    // Parse the struct
    let item: ItemStruct = parse2(input)?;
    let struct_name = &item.ident;

    // Determine the scope based on provided_in
    let scope = match attr_args.provided_in.as_deref() {
        Some("root") | None => quote! { ferric_core::di::Scope::Singleton },
        Some("any") => quote! { ferric_core::di::Scope::Transient },
        Some("platform") => quote! { ferric_core::di::Scope::Singleton },
        Some(other) => {
            return Err(syn::Error::new_spanned(
                &item.ident,
                format!("Unknown provided_in value: '{}'. Use 'root', 'any', or 'platform'.", other),
            ));
        }
    };

    // Collect fields that need injection
    let inject_fields = collect_inject_fields(&item.fields)?;

    // Generate the field initialization
    let field_inits: Vec<_> = if inject_fields.is_empty() {
        // No inject fields, try to use Default
        vec![quote! { ..Default::default() }]
    } else {
        inject_fields
            .iter()
            .map(|(name, ty)| {
                quote! {
                    #name: injector.resolve::<#ty>()
                        .expect(concat!("Failed to resolve dependency: ", stringify!(#ty)))
                        .as_ref()
                        .clone()
                }
            })
            .collect()
    };

    // Check if there are non-inject fields
    let has_other_fields = match &item.fields {
        Fields::Named(named) => named.named.len() > inject_fields.len(),
        _ => false,
    };

    let create_impl = if inject_fields.is_empty() {
        // Use Default trait
        quote! {
            fn create(_injector: &ferric_core::di::Injector) -> Self {
                Self::default()
            }
        }
    } else if has_other_fields {
        // Some fields need injection, others use default
        let inject_inits: Vec<_> = inject_fields
            .iter()
            .map(|(name, ty)| {
                quote! {
                    #name: {
                        let rc = injector.resolve::<#ty>()
                            .expect(concat!("Failed to resolve: ", stringify!(#ty)));
                        (*rc).clone()
                    }
                }
            })
            .collect();

        quote! {
            fn create(injector: &ferric_core::di::Injector) -> Self {
                Self {
                    #(#inject_inits,)*
                    ..Default::default()
                }
            }
        }
    } else {
        // All fields need injection
        let inject_inits: Vec<_> = inject_fields
            .iter()
            .map(|(name, ty)| {
                quote! {
                    #name: {
                        let rc = injector.resolve::<#ty>()
                            .expect(concat!("Failed to resolve: ", stringify!(#ty)));
                        (*rc).clone()
                    }
                }
            })
            .collect();

        quote! {
            fn create(injector: &ferric_core::di::Injector) -> Self {
                Self {
                    #(#inject_inits,)*
                }
            }
        }
    };

    let expanded = quote! {
        #item

        impl ferric_core::di::Injectable for #struct_name {
            #create_impl
        }

        impl #struct_name {
            /// Get the default scope for this service.
            #[doc(hidden)]
            pub fn __default_scope() -> ferric_core::di::Scope {
                #scope
            }

            /// Register this service with the given injector.
            pub fn register(injector: &ferric_core::di::Injector) {
                injector.register::<Self>(Self::__default_scope());
            }
        }
    };

    Ok(expanded)
}

/// Implement the derive(Injectable) macro.
pub fn derive_injectable_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let struct_name = &input.ident;

    // Collect inject fields
    let inject_fields = match &input.data {
        syn::Data::Struct(data) => collect_inject_fields(&data.fields)?,
        _ => return Err(syn::Error::new_spanned(
            &input.ident,
            "Injectable can only be derived for structs",
        )),
    };

    let create_impl = if inject_fields.is_empty() {
        // No inject fields, use Default
        quote! {
            fn create(_injector: &ferric_core::di::Injector) -> Self {
                Self::default()
            }
        }
    } else {
        // Generate injection for each field
        let inject_inits: Vec<_> = inject_fields
            .iter()
            .map(|(name, ty)| {
                quote! {
                    #name: {
                        let rc = injector.resolve::<#ty>()
                            .expect(concat!("Failed to resolve: ", stringify!(#ty)));
                        (*rc).clone()
                    }
                }
            })
            .collect();

        // Check if there are other fields
        let all_fields_injected = match &input.data {
            syn::Data::Struct(data) => match &data.fields {
                Fields::Named(named) => named.named.len() == inject_fields.len(),
                _ => true,
            },
            _ => true,
        };

        if all_fields_injected {
            quote! {
                fn create(injector: &ferric_core::di::Injector) -> Self {
                    Self {
                        #(#inject_inits,)*
                    }
                }
            }
        } else {
            quote! {
                fn create(injector: &ferric_core::di::Injector) -> Self {
                    Self {
                        #(#inject_inits,)*
                        ..Default::default()
                    }
                }
            }
        }
    };

    let expanded = quote! {
        impl ferric_core::di::Injectable for #struct_name {
            #create_impl
        }
    };

    Ok(expanded)
}

/// Collect fields marked with #[inject].
fn collect_inject_fields(fields: &Fields) -> syn::Result<Vec<(Ident, Type)>> {
    let mut inject_fields = Vec::new();

    if let Fields::Named(named) = fields {
        for field in &named.named {
            let has_inject = field.attrs.iter().any(|attr| attr.path().is_ident("inject"));

            if has_inject {
                let name = field.ident.clone().unwrap();
                let ty = field.ty.clone();
                inject_fields.push((name, ty));
            }
        }
    }

    Ok(inject_fields)
}

