//! #[derive(Validate)] implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, Fields, Ident, Result, Type};
use darling::FromField;

#[derive(Debug, FromField)]
#[darling(attributes(validate))]
struct ValidateField {
    ident: Option<Ident>,
    // Type is available but not currently used for validation
    // Could be used for type-specific validators in the future
    #[allow(dead_code)]
    ty: Type,

    #[darling(default)]
    required: bool,

    #[darling(default)]
    email: bool,

    #[darling(default)]
    min_length: Option<usize>,

    #[darling(default)]
    max_length: Option<usize>,

    #[darling(default)]
    min: Option<i64>,

    #[darling(default)]
    max: Option<i64>,

    #[darling(default)]
    pattern: Option<String>,

    /// Custom validation function name
    #[darling(default)]
    custom: Option<String>,
}

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => return Err(syn::Error::new_spanned(
                &input,
                "Validate can only be derived for structs with named fields"
            )),
        },
        _ => return Err(syn::Error::new_spanned(
            &input,
            "Validate can only be derived for structs"
        )),
    };

    let mut field_validations = Vec::new();

    for field in fields {
        let field_info = ValidateField::from_field(field)?;
        let field_name = field_info.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let mut validators = Vec::new();

        if field_info.required {
            validators.push(quote! {
                if self.#field_name.to_string().trim().is_empty() {
                    errors.insert(
                        #field_name_str.to_string(),
                        ::ferric_forms::validators::ValidationError::new("This field is required")
                    );
                }
            });
        }

        if field_info.email {
            validators.push(quote! {
                {
                    let email_regex = ::regex::Regex::new(
                        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
                    ).unwrap();
                    let value = self.#field_name.to_string();
                    if !value.is_empty() && !email_regex.is_match(&value) {
                        errors.insert(
                            #field_name_str.to_string(),
                            ::ferric_forms::validators::ValidationError::new("Please enter a valid email address")
                        );
                    }
                }
            });
        }

        if let Some(min_len) = field_info.min_length {
            validators.push(quote! {
                {
                    let value = self.#field_name.to_string();
                    if value.len() < #min_len {
                        errors.insert(
                            #field_name_str.to_string(),
                            ::ferric_forms::validators::ValidationError::new(
                                &format!("Minimum length is {}", #min_len)
                            )
                        );
                    }
                }
            });
        }

        if let Some(max_len) = field_info.max_length {
            validators.push(quote! {
                {
                    let value = self.#field_name.to_string();
                    if value.len() > #max_len {
                        errors.insert(
                            #field_name_str.to_string(),
                            ::ferric_forms::validators::ValidationError::new(
                                &format!("Maximum length is {}", #max_len)
                            )
                        );
                    }
                }
            });
        }

        if let Some(min_val) = field_info.min {
            validators.push(quote! {
                {
                    let value = self.#field_name as i64;
                    if value < #min_val {
                        errors.insert(
                            #field_name_str.to_string(),
                            ::ferric_forms::validators::ValidationError::new(
                                &format!("Value must be at least {}", #min_val)
                            )
                        );
                    }
                }
            });
        }

        if let Some(max_val) = field_info.max {
            validators.push(quote! {
                {
                    let value = self.#field_name as i64;
                    if value > #max_val {
                        errors.insert(
                            #field_name_str.to_string(),
                            ::ferric_forms::validators::ValidationError::new(
                                &format!("Value must be at most {}", #max_val)
                            )
                        );
                    }
                }
            });
        }

        if let Some(pattern) = &field_info.pattern {
            validators.push(quote! {
                {
                    let pattern_regex = ::regex::Regex::new(#pattern).unwrap();
                    let value = self.#field_name.to_string();
                    if !value.is_empty() && !pattern_regex.is_match(&value) {
                        errors.insert(
                            #field_name_str.to_string(),
                            ::ferric_forms::validators::ValidationError::new(
                                "Value does not match the required pattern"
                            )
                        );
                    }
                }
            });
        }

        // Custom validation function
        if let Some(custom_fn) = &field_info.custom {
            let custom_fn_ident = syn::parse_str::<Ident>(custom_fn)
                .unwrap_or_else(|_| panic!("Invalid custom validator function name: {}", custom_fn));

            validators.push(quote! {
                {
                    // Call the custom validation function
                    // Function should have signature: fn(&FieldType) -> Result<(), String>
                    if let Err(msg) = self.#custom_fn_ident(&self.#field_name) {
                        errors.insert(
                            #field_name_str.to_string(),
                            ::ferric_forms::validators::ValidationError::new(&msg)
                        );
                    }
                }
            });
        }

        if !validators.is_empty() {
            field_validations.push(quote! {
                #(#validators)*
            });
        }
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Validate this form and return any validation errors.
            pub fn validate(&self) -> ::ferric_forms::validators::ValidationResult {
                let mut errors = ::ferric_forms::validators::ValidationErrors::new();

                #(#field_validations)*

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }

            /// Check if this form is valid.
            pub fn is_valid(&self) -> bool {
                self.validate().is_ok()
            }

            /// Get validation errors for a specific field.
            pub fn field_error(&self, field: &str) -> Option<String> {
                self.validate()
                    .err()
                    .and_then(|errors| errors.get(field).map(|e| e.message.clone()))
            }
        }
    };

    Ok(expanded)
}

