//! Host listener and host binding macro implementations.

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{parse2, ImplItemFn, LitStr};

use crate::utils;

/// Implement the #[host_listener] attribute macro.
pub fn host_listener_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse the arguments (event name and optional $event)
    let args_str = args.to_string();
    let parts: Vec<&str> = args_str.split(',').map(|s| s.trim().trim_matches('"')).collect();

    let event_name = parts.first()
        .ok_or_else(|| syn::Error::new_spanned(&args, "Expected event name"))?
        .trim_matches('"');

    let pass_event = parts.get(1).map(|s| s.contains("$event")).unwrap_or(false);

    // Parse the function
    let method: ImplItemFn = parse2(input)?;
    let method_name = &method.sig.ident;
    let method_vis = &method.vis;
    let method_attrs = &method.attrs;
    let method_block = &method.block;
    let method_inputs = &method.sig.inputs;
    let method_output = &method.sig.output;

    // Parse event name for modifiers (e.g., "keydown.enter")
    let (event, modifier) = utils::parse_event(event_name);
    let event_lit = utils::lit_str(&event);

    // Generate the modifier check if present
    let modifier_check = if let Some(mod_key) = modifier {
        let mod_lit = utils::lit_str(&mod_key);
        quote! {
            // Check modifier key
            if let Some(key_event) = event.dyn_ref::<web_sys::KeyboardEvent>() {
                if key_event.key().to_lowercase() != #mod_lit {
                    return;
                }
            }
        }
    } else {
        quote! {}
    };

    // Generate the event handler registration
    let handler_setup = if pass_event {
        quote! {
            let callback = move |event: web_sys::Event| {
                #modifier_check
                self_clone.#method_name(event);
            };
        }
    } else {
        quote! {
            let callback = move |_event: web_sys::Event| {
                #modifier_check
                self_clone.#method_name();
            };
        }
    };

    // Create the register function name
    let register_fn_name = format_ident!("__register_host_listener_{}", method_name);

    let expanded = quote! {
        #(#method_attrs)*
        #method_vis fn #method_name #method_inputs #method_output #method_block

        #[doc(hidden)]
        fn #register_fn_name(&self, element: &web_sys::Element) {
            use wasm_bindgen::JsCast;

            let self_clone = self.clone();
            #handler_setup

            let closure = wasm_bindgen::closure::Closure::wrap(
                Box::new(callback) as Box<dyn Fn(web_sys::Event)>
            );

            element.add_event_listener_with_callback(
                #event_lit,
                closure.as_ref().unchecked_ref()
            ).expect("Failed to add event listener");

            closure.forget(); // Prevent closure from being dropped
        }
    };

    Ok(expanded)
}

/// Implement the #[host_binding] attribute macro.
pub fn host_binding_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    // Parse the binding string
    let binding = if let Ok(lit) = parse2::<LitStr>(args.clone()) {
        lit.value()
    } else {
        args.to_string().trim_matches('"').to_string()
    };

    let binding_lit = utils::lit_str(&binding);

    // For field attributes, just pass through with documentation
    // The component macro will process these annotations later
    let expanded = quote! {
        #[doc = concat!("Host binding: ", #binding_lit)]
        #input
    };

    Ok(expanded)
}

