//! Event handling macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Expr, Ident, Result, Token};
use syn::parse::{Parse, ParseStream};

/// Event handler specification.
struct EventHandler {
    event_name: Ident,
    _arrow: Token![=>],
    handler: Expr,
}

impl Parse for EventHandler {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            event_name: input.parse()?,
            _arrow: input.parse()?,
            handler: input.parse()?,
        })
    }
}

/// Implementation for the `on!` macro for event handlers.
pub fn on_impl(input: TokenStream) -> Result<TokenStream> {
    let handler: EventHandler = parse2(input)?;
    let event_name = handler.event_name;
    let handler_expr = handler.handler;

    Ok(quote! {
        {
            let event_name = stringify!(#event_name);
            let handler = #handler_expr;
            ::ferric_core::events::EventListener::new(event_name, handler)
        }
    })
}

/// Implementation for the `emit!` macro.
pub fn emit_impl(input: TokenStream) -> Result<TokenStream> {
    let expr: Expr = parse2(input)?;

    Ok(quote! {
        {
            let emitter = #expr;
            emitter.emit()
        }
    })
}

