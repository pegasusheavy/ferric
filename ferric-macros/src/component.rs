//! Component macro implementation.

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, DeriveInput, ItemStruct, Fields, Ident, Type};

use crate::utils;

/// Arguments for the #[component] attribute.
#[derive(Debug, FromMeta)]
pub struct ComponentArgs {
    /// CSS selector for the component.
    selector: String,

    /// Inline template string.
    #[darling(default)]
    template: Option<String>,

    /// Path to external template file.
    #[darling(default)]
    template_url: Option<String>,

    /// Inline styles (can be single string or array).
    #[darling(default)]
    styles: Option<String>,

    /// Paths to external stylesheets.
    #[darling(default, multiple)]
    style_urls: Vec<String>,

    /// View encapsulation mode.
    #[darling(default)]
    encapsulation: Option<String>,

    /// Change detection strategy.
    #[darling(default)]
    change_detection: Option<String>,
}

/// Implement the #[component] attribute macro.
pub fn component_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse the attribute arguments
    let attr_args = darling::ast::NestedMeta::parse_meta_list(args)?;
    let args = ComponentArgs::from_list(&attr_args)?;

    // Parse the struct
    let item: ItemStruct = parse2(input)?;

    // Validate: must have either template or template_url
    if args.template.is_none() && args.template_url.is_none() {
        return Err(syn::Error::new_spanned(
            &item.ident,
            "Component must have either 'template' or 'template_url'",
        ));
    }

    let struct_name = &item.ident;
    let selector = &args.selector;

    // Get template content
    let template = args.template.as_deref().unwrap_or("");
    let template_lit = utils::lit_str(template);

    // Get styles - combine inline styles with style_urls content
    let mut combined_styles = args.styles.clone().unwrap_or_default();

    // Load external stylesheets
    for style_url in &args.style_urls {
        match std::fs::read_to_string(style_url) {
            Ok(content) => {
                if !combined_styles.is_empty() {
                    combined_styles.push_str("\n\n");
                }
                combined_styles.push_str(&content);
            }
            Err(e) => {
                return Err(syn::Error::new_spanned(
                    &item.ident,
                    format!("Failed to read style file '{}': {}", style_url, e),
                ));
            }
        }
    }

    let styles_lit = utils::lit_str(&combined_styles);

    // Determine encapsulation
    let encapsulation = match args.encapsulation.as_deref() {
        Some("ShadowDom") => quote! { ferric_core::component::ViewEncapsulation::ShadowDom },
        Some("None") => quote! { ferric_core::component::ViewEncapsulation::None },
        _ => quote! { ferric_core::component::ViewEncapsulation::Emulated },
    };

    // Determine change detection
    let change_detection = match args.change_detection.as_deref() {
        Some("OnPush") => quote! { ferric_core::component::ChangeDetectionStrategy::OnPush },
        _ => quote! { ferric_core::component::ChangeDetectionStrategy::Default },
    };

    // Collect input and output fields
    let (inputs, outputs) = collect_io_fields(&item.fields)?;

    // Generate the input names
    let input_names: Vec<_> = inputs.iter().map(|(name, _, _)| utils::lit_str(name)).collect();
    let output_names: Vec<_> = outputs.iter().map(|(name, _)| utils::lit_str(name)).collect();

    // Generate style_urls literals
    let style_url_lits: Vec<_> = args.style_urls.iter().map(|url| utils::lit_str(url)).collect();

    // Add a hidden field to store component metadata
    let _metadata_field = quote! {
        #[doc(hidden)]
        __ferric_metadata: ::std::marker::PhantomData<()>,
    };

    // Generate the implementation
    let expanded = quote! {
        #item

        impl ferric_core::component::Component for #struct_name {
            fn selector(&self) -> &'static str {
                #selector
            }

            fn template(&self) -> &str {
                #template_lit
            }

            fn styles(&self) -> Option<&str> {
                let s = #styles_lit;
                if s.is_empty() { None } else { Some(s) }
            }

            fn render(&self) -> Result<web_sys::Element, String> {
                // Compile and render the template
                let document = ferric_core::dom::document();
                let compiled = ferric_core::template::compile_template(self.template())?;
                let fragment = compiled.create_fragment(&document)
                    .map_err(|e| format!("{:?}", e))?;

                // Create wrapper element
                let wrapper = document.create_element("div")
                    .map_err(|e| format!("{:?}", e))?;
                wrapper.append_child(&fragment)
                    .map_err(|e| format!("{:?}", e))?;

                Ok(wrapper)
            }
        }

        impl ferric_core::lifecycle::Lifecycle for #struct_name {}

        impl #struct_name {
            /// Get the component metadata.
            #[doc(hidden)]
            pub fn __component_metadata() -> ferric_core::component::ComponentMetadata {
                ferric_core::component::ComponentMetadata {
                    selector: #selector.to_string(),
                    template: Some(#template_lit.to_string()),
                    template_url: None,
                    styles: vec![#styles_lit.to_string()],
                    style_urls: vec![#(#style_url_lits.to_string()),*],
                    inputs: vec![#(#input_names.to_string()),*],
                    outputs: vec![#(#output_names.to_string()),*],
                    providers: vec![],
                    encapsulation: #encapsulation,
                    change_detection: #change_detection,
                }
            }
        }
    };

    Ok(expanded)
}

/// Implement the derive(Component) macro.
pub fn derive_component_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let struct_name = &input.ident;

    // Look for #[component(...)] attribute
    let component_attr = input.attrs.iter().find(|attr| attr.path().is_ident("component"));

    let (selector, template) = if let Some(attr) = component_attr {
        // Parse component attribute
        let meta = attr.meta.clone();
        if let syn::Meta::List(list) = meta {
            let args = darling::ast::NestedMeta::parse_meta_list(list.tokens)?;
            let parsed = ComponentArgs::from_list(&args)?;
            (parsed.selector, parsed.template.unwrap_or_default())
        } else {
            return Err(syn::Error::new_spanned(
                attr,
                "Expected #[component(selector = \"...\", template = \"...\")]",
            ));
        }
    } else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "Missing #[component] attribute. Add #[component(selector = \"...\")] to the struct.",
        ));
    };

    let selector_lit = utils::lit_str(&selector);
    let template_lit = utils::lit_str(&template);

    let expanded = quote! {
        impl ferric_core::component::Component for #struct_name {
            fn selector(&self) -> &'static str {
                #selector_lit
            }

            fn template(&self) -> &str {
                #template_lit
            }

            fn render(&self) -> Result<web_sys::Element, String> {
                let document = ferric_core::dom::document();
                let compiled = ferric_core::template::compile_template(self.template())?;
                let fragment = compiled.create_fragment(&document)
                    .map_err(|e| format!("{:?}", e))?;

                let wrapper = document.create_element("div")
                    .map_err(|e| format!("{:?}", e))?;
                wrapper.append_child(&fragment)
                    .map_err(|e| format!("{:?}", e))?;

                Ok(wrapper)
            }
        }

        impl ferric_core::lifecycle::Lifecycle for #struct_name {}
    };

    Ok(expanded)
}

/// Collect input and output fields from struct fields.
type InputFields = Vec<(String, Ident, Type)>;
type OutputFields = Vec<(String, Ident)>;

fn collect_io_fields(fields: &Fields) -> syn::Result<(InputFields, OutputFields)> {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    if let Fields::Named(named) = fields {
        for field in &named.named {
            let field_name = field.ident.as_ref().unwrap();

            for attr in &field.attrs {
                if attr.path().is_ident("input") {
                    // Check for alias
                    let alias = extract_alias(attr)?;
                    let name = alias.unwrap_or_else(|| field_name.to_string());
                    inputs.push((name, field_name.clone(), field.ty.clone()));
                } else if attr.path().is_ident("output") {
                    let alias = extract_alias(attr)?;
                    let name = alias.unwrap_or_else(|| field_name.to_string());
                    outputs.push((name, field_name.clone()));
                }
            }
        }
    }

    Ok((inputs, outputs))
}

/// Extract the alias from an input/output attribute.
fn extract_alias(attr: &syn::Attribute) -> syn::Result<Option<String>> {
    // Handle #[input] without arguments
    if matches!(attr.meta, syn::Meta::Path(_)) {
        return Ok(None);
    }

    // Handle #[input(alias = "...")]
    if let syn::Meta::List(list) = &attr.meta {
        let nested: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]> =
            list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;

        for meta in nested {
            if let syn::Meta::NameValue(nv) = meta
                && nv.path.is_ident("alias")
                    && let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = nv.value {
                        return Ok(Some(s.value()));
                    }
        }
    }

    Ok(None)
}

