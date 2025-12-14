//! Utilities for converting IDB requests to futures.

use crate::error::{IdbError, IdbResult};
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbOpenDbRequest, IdbRequest};

/// Convert an IdbRequest to a Future.
pub async fn request_to_future(request: &IdbRequest) -> IdbResult<JsValue> {
    let promise = Promise::new(&mut |resolve, reject| {
        let on_success = Closure::once(Box::new(move |_event: web_sys::Event| {
            resolve.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        let on_error = Closure::once(Box::new(move |_event: web_sys::Event| {
            reject.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        on_success.forget();
        on_error.forget();
    });

    JsFuture::from(promise).await?;

    request.result().map_err(IdbError::from)
}

/// Convert an IdbOpenDbRequest to a Future, with upgrade handler.
pub async fn open_request_to_future<F>(
    request: &IdbOpenDbRequest,
    on_upgrade: Option<F>,
) -> IdbResult<web_sys::IdbDatabase>
where
    F: FnOnce(&web_sys::IdbDatabase, u32, u32) -> IdbResult<()> + 'static,
{
    // Set up upgrade handler first (before success/error)
    let upgrade_cell = std::cell::RefCell::new(on_upgrade);

    if upgrade_cell.borrow().is_some() {
        let request_clone = request.clone();
        let on_upgrade_needed = Closure::once(Box::new(move |event: web_sys::IdbVersionChangeEvent| {
            if let Some(upgrade_fn) = upgrade_cell.borrow_mut().take()
                && let Ok(Some(db)) = request_clone.result().map(|r| r.dyn_into::<web_sys::IdbDatabase>().ok()) {
                    let old_version = event.old_version() as u32;
                    let new_version = event.new_version().unwrap_or(0.0) as u32;

                    if let Err(e) = upgrade_fn(&db, old_version, new_version) {
                        web_sys::console::error_1(&format!("Upgrade error: {:?}", e).into());
                    }
                }
        }) as Box<dyn FnOnce(_)>);

        request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        on_upgrade_needed.forget();
    }

    let promise = Promise::new(&mut |resolve, reject| {
        // Success handler
        let resolve_clone = resolve.clone();
        let on_success = Closure::once(Box::new(move |_event: web_sys::Event| {
            resolve_clone.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        // Error handler
        let reject_clone = reject.clone();
        let on_error = Closure::once(Box::new(move |_event: web_sys::Event| {
            reject_clone.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        // Blocked handler
        let on_blocked = Closure::once(Box::new(move |_event: web_sys::Event| {
            web_sys::console::warn_1(&"Database upgrade blocked - close other tabs".into());
        }) as Box<dyn FnOnce(_)>);

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        request.set_onblocked(Some(on_blocked.as_ref().unchecked_ref()));

        on_success.forget();
        on_error.forget();
        on_blocked.forget();
    });

    JsFuture::from(promise).await?;

    request
        .result()
        .map_err(IdbError::from)?
        .dyn_into::<web_sys::IdbDatabase>()
        .map_err(|_| IdbError::OpenFailed("Failed to cast to IdbDatabase".to_string()))
}

/// Create a promise that resolves when a transaction completes.
pub async fn transaction_complete(transaction: &web_sys::IdbTransaction) -> IdbResult<()> {
    let promise = Promise::new(&mut |resolve, reject| {
        let resolve_clone = resolve.clone();
        let on_complete = Closure::once(Box::new(move |_event: web_sys::Event| {
            resolve_clone.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        let reject_clone = reject.clone();
        let on_error = Closure::once(Box::new(move |_event: web_sys::Event| {
            reject_clone.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        let on_abort = Closure::once(Box::new(move |_event: web_sys::Event| {
            reject.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        transaction.set_oncomplete(Some(on_complete.as_ref().unchecked_ref()));
        transaction.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        transaction.set_onabort(Some(on_abort.as_ref().unchecked_ref()));

        on_complete.forget();
        on_error.forget();
        on_abort.forget();
    });

    JsFuture::from(promise).await.map_err(IdbError::from)?;
    Ok(())
}

