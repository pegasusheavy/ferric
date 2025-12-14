//! Form builder macro for typed forms.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, DeriveInput, Data, Fields, Result};

/// Implementation for the `TypedForm` derive macro.
pub fn typed_form_impl(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let name = &input.ident;
    let form_name = quote::format_ident!("{}Form", name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => return Err(syn::Error::new_spanned(
                input,
                "TypedForm can only be derived for structs with named fields",
            )),
        },
        _ => return Err(syn::Error::new_spanned(
            input,
            "TypedForm can only be derived for structs",
        )),
    };

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let form_fields = field_names.iter().zip(field_types.iter()).map(|(name, ty)| {
        quote! {
            pub #name: ::ferric_forms::FormControl<#ty>
        }
    });

    let field_inits = field_names.iter().map(|name| {
        quote! {
            #name: ::ferric_forms::FormControl::new(value.#name)
        }
    });

    let value_extraction = field_names.iter().map(|name| {
        quote! {
            #name: form.#name.value()
        }
    });

    Ok(quote! {
        /// Auto-generated typed form for #name
        pub struct #form_name {
            #(#form_fields),*
        }

        impl #form_name {
            /// Create a new form from a value.
            pub fn from_value(value: #name) -> Self {
                Self {
                    #(#field_inits),*
                }
            }

            /// Extract the value from the form.
            pub fn value(&self) -> #name {
                #name {
                    #(#value_extraction),*
                }
            }

            /// Check if the form is valid.
            pub fn is_valid(&self) -> bool {
                true #(&& self.#field_names.is_valid())*
            }
        }

        impl From<#name> for #form_name {
            fn from(value: #name) -> Self {
                Self::from_value(value)
            }
        }

        impl From<#form_name> for #name {
            fn from(form: #form_name) -> Self {
                form.value()
            }
        }
    })
}

