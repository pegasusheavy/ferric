//! HTTP Interceptor decorator implementation (Angular HttpInterceptor equivalent).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct, Result};
use darling::FromMeta;

/// Arguments for the #[interceptor] decorator.
#[derive(Debug, FromMeta, Default)]
struct InterceptorArgs {
    /// Priority (higher = earlier in chain)
    #[darling(default)]
    priority: Option<i32>,

    /// Only intercept specific paths
    #[darling(default)]
    paths: Option<syn::Expr>,
}

/// Implementation for #[interceptor] decorator.
pub fn interceptor_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let item: ItemStruct = parse2(input.clone())?;
    let name = &item.ident;

    let interceptor_args = InterceptorArgs::default();

    let priority = interceptor_args.priority.unwrap_or(0);
    let paths = interceptor_args.paths.as_ref()
        .map(|p| quote! { Some(#p) })
        .unwrap_or_else(|| quote! { None });

    Ok(quote! {
        #item

        impl ::ferric_http::Interceptor for #name {
            fn intercept(
                &self,
                request: ::ferric_http::Request,
                next: Box<dyn Fn(::ferric_http::Request) -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Result<::ferric_http::Response, ::ferric_http::Error>> + Send>> + Send + Sync>,
            ) -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Result<::ferric_http::Response, ::ferric_http::Error>> + Send>> {
                self.handle(request, next)
            }

            fn priority(&self) -> i32 {
                #priority
            }

            fn should_intercept(&self, request: &::ferric_http::Request) -> bool {
                if let Some(paths) = #paths {
                    paths.iter().any(|path: &str| request.url().contains(path))
                } else {
                    true
                }
            }
        }
    })
}

