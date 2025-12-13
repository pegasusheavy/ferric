//! Component file inclusion macro.
//!
//! Provides `include_template!` and `include_styles!` macros for cleaner syntax,
//! plus enhances `#[component]` to support `template_url` and `style_urls`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Result, Token};

/// Arguments for include_template! macro
pub struct IncludeTemplateArgs {
    pub path: LitStr,
}

impl Parse for IncludeTemplateArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let path: LitStr = input.parse()?;
        Ok(Self { path })
    }
}

/// Expand include_template! macro
pub fn expand_include_template(input: TokenStream) -> Result<TokenStream> {
    let args: IncludeTemplateArgs = syn::parse2(input)?;
    let path = &args.path;

    Ok(quote! {
        include_str!(#path)
    })
}

/// Expand include_styles! macro
pub fn expand_include_styles(input: TokenStream) -> Result<TokenStream> {
    let args: IncludeTemplateArgs = syn::parse2(input)?;
    let path = &args.path;

    Ok(quote! {
        include_str!(#path)
    })
}

/// Arguments for component_files! macro
/// Usage: component_files!("template.html", "styles.css")
pub struct ComponentFilesArgs {
    pub template_path: LitStr,
    pub styles_path: Option<LitStr>,
}

impl Parse for ComponentFilesArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let template_path: LitStr = input.parse()?;

        let styles_path = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { template_path, styles_path })
    }
}

/// Expand component_files! macro
pub fn expand_component_files(input: TokenStream) -> Result<TokenStream> {
    let args: ComponentFilesArgs = syn::parse2(input)?;
    let template_path = &args.template_path;

    let styles = if let Some(ref styles_path) = args.styles_path {
        quote! {
            pub const STYLES: &str = include_str!(#styles_path);
        }
    } else {
        quote! {
            pub const STYLES: &str = "";
        }
    };

    Ok(quote! {
        pub const TEMPLATE: &str = include_str!(#template_path);
        #styles
    })
}

