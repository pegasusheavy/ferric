//! # Shopping Cart Demo
//!
//! Demonstrates complex state management with:
//! - Multiple interconnected signals
//! - Computed values that depend on collections
//! - `watch()` for observing specific changes

use ferric_core::reactive::{computed, effect, signal, watch, Computed, Effect, Signal};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, Element};

// Load template and styles at compile time
const TEMPLATE: &str = include_str!("../templates/shopping_cart.html");
const STYLES: &str = include_str!("../styles/shopping_cart.css");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product: Product,
    pub quantity: u32,
}

pub struct ShoppingCartDemo {
    cart: Signal<Vec<CartItem>>,
    discount_code: Signal<Option<String>>,
    item_count: Computed<u32>,
    subtotal: Computed<f64>,
    discount: Computed<f64>,
    total: Computed<f64>,
    is_empty: Computed<bool>,
    _effects: Vec<Effect>,
}

impl ShoppingCartDemo {
    fn products() -> Vec<Product> {
        vec![
            Product { id: 1, name: "Rust Book".to_string(), price: 39.99 },
            Product { id: 2, name: "WASM Tutorial".to_string(), price: 24.99 },
            Product { id: 3, name: "Ferric Framework".to_string(), price: 0.00 },
            Product { id: 4, name: "Coffee Mug".to_string(), price: 12.99 },
            Product { id: 5, name: "Laptop Stickers".to_string(), price: 5.99 },
        ]
    }

    fn get_discount_rate(code: &str) -> f64 {
        match code.to_uppercase().as_str() {
            "RUST10" => 0.10,
            "FERRIC20" => 0.20,
            "WASM50" => 0.50,
            _ => 0.0,
        }
    }

    pub fn mount(container: &Element) -> Result<(), JsValue> {
        inject_styles("shopping-cart-styles", STYLES);

        let cart: Signal<Vec<CartItem>> = signal(vec![]);
        let discount_code: Signal<Option<String>> = signal(None);

        let cart_for_count = cart.clone();
        let item_count = computed(move || {
            cart_for_count.get().iter().map(|i| i.quantity).sum()
        });

        let cart_for_subtotal = cart.clone();
        let subtotal = computed(move || {
            cart_for_subtotal.get().iter()
                .map(|i| i.product.price * i.quantity as f64)
                .sum()
        });

        let discount_code_for_discount = discount_code.clone();
        let subtotal_for_discount = subtotal.clone();
        let discount = computed(move || {
            let sub = subtotal_for_discount.get();
            if let Some(code) = discount_code_for_discount.get() {
                let rate = Self::get_discount_rate(&code);
                sub * rate
            } else {
                0.0
            }
        });

        let subtotal_for_total = subtotal.clone();
        let discount_for_total = discount.clone();
        let total = computed(move || {
            subtotal_for_total.get() - discount_for_total.get()
        });

        let cart_for_empty = cart.clone();
        let is_empty = computed(move || cart_for_empty.get().is_empty());

        // Render template
        container.set_inner_html(TEMPLATE);

        // Render products list
        Self::render_products(container);

        // Effect: Update cart display
        let container_for_cart = container.clone();
        let cart_for_effect = cart.clone();
        let cart_effect = effect(move || {
            let items = cart_for_effect.get();
            Self::render_cart_items(&container_for_cart, &items);
        });

        // Effect: Update totals display
        let container_for_totals = container.clone();
        let item_count_for_effect = item_count.clone();
        let subtotal_for_effect = subtotal.clone();
        let discount_for_effect = discount.clone();
        let total_for_effect = total.clone();
        let totals_effect = effect(move || {
            let count = item_count_for_effect.get();
            let sub = subtotal_for_effect.get();
            let disc = discount_for_effect.get();
            let tot = total_for_effect.get();

            Self::update_totals(&container_for_totals, count, sub, disc, tot);
        });

        // Watch: Log when cart changes
        let cart_for_watch = cart.clone();
        let watch_effect = watch(
            move || cart_for_watch.get().len(),
            |new_len, old_len| {
                console::log_1(&format!(
                    "🛒 Cart changed: {} → {} items",
                    old_len.unwrap_or(0),
                    new_len
                ).into());
            }
        );

        // Set up product buttons
        for product in Self::products() {
            Self::setup_add_button(container, &cart, product);
        }

        Self::setup_discount_input(container, &discount_code);
        Self::setup_clear_button(container, &cart, &discount_code);

        std::mem::forget(ShoppingCartDemo {
            cart,
            discount_code,
            item_count,
            subtotal,
            discount,
            total,
            is_empty,
            _effects: vec![cart_effect, totals_effect, watch_effect],
        });

        Ok(())
    }

    fn render_products(container: &Element) {
        if let Some(list) = container.query_selector("#products-list").ok().flatten() {
            let html: String = Self::products()
                .iter()
                .map(|p| format!(
                    r#"<div class="product-item">
                        <span class="product-name">{}</span>
                        <span class="product-price">${:.2}</span>
                        <button class="add-to-cart btn btn-success btn-small" data-product-id="{}">Add</button>
                    </div>"#,
                    p.name, p.price, p.id
                ))
                .collect();
            list.set_inner_html(&html);
        }
    }

    fn render_cart_items(container: &Element, items: &[CartItem]) {
        if let Some(cart_el) = container.query_selector("#cart-items").ok().flatten() {
            if items.is_empty() {
                cart_el.set_inner_html(r#"<p class="empty-message">Your cart is empty</p>"#);
            } else {
                let html: String = items.iter().map(|item| format!(
                    r#"<div class="cart-item">
                        <span class="item-name">{}</span>
                        <span class="item-qty">×{}</span>
                        <span class="item-price">${:.2}</span>
                    </div>"#,
                    item.product.name,
                    item.quantity,
                    item.product.price * item.quantity as f64
                )).collect();
                cart_el.set_inner_html(&html);
            }
        }

        if let Some(badge) = container.query_selector("#cart-badge").ok().flatten() {
            let count: u32 = items.iter().map(|i| i.quantity).sum();
            if count > 0 {
                badge.set_text_content(Some(&format!("{}", count)));
                let _ = badge.class_list().remove_1("hidden");
            } else {
                badge.set_text_content(None);
                let _ = badge.class_list().add_1("hidden");
            }
        }
    }

    fn update_totals(container: &Element, _count: u32, subtotal: f64, discount: f64, total: f64) {
        if let Some(el) = container.query_selector("#subtotal").ok().flatten() {
            el.set_text_content(Some(&format!("${:.2}", subtotal)));
        }
        if let Some(el) = container.query_selector("#discount").ok().flatten() {
            el.set_text_content(Some(&format!("-${:.2}", discount)));
        }
        if let Some(row) = container.query_selector("#discount-row").ok().flatten() {
            if discount > 0.0 {
                let _ = row.class_list().remove_1("hidden");
            } else {
                let _ = row.class_list().add_1("hidden");
            }
        }
        if let Some(el) = container.query_selector("#total").ok().flatten() {
            el.set_text_content(Some(&format!("${:.2}", total)));
        }
    }

    fn setup_add_button(container: &Element, cart: &Signal<Vec<CartItem>>, product: Product) {
        let selector = format!(".add-to-cart[data-product-id='{}']", product.id);
        if let Some(button) = container.query_selector(&selector).ok().flatten() {
            let cart = cart.clone();
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                cart.mutate(|items| {
                    if let Some(item) = items.iter_mut().find(|i| i.product.id == product.id) {
                        item.quantity += 1;
                    } else {
                        items.push(CartItem {
                            product: product.clone(),
                            quantity: 1,
                        });
                    }
                });
            }) as Box<dyn Fn(_)>);

            let _ = button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }

    fn setup_discount_input(container: &Element, discount_code: &Signal<Option<String>>) {
        if let Some(button) = container.query_selector("#apply-discount").ok().flatten() {
            let discount_code = discount_code.clone();
            let container = container.clone();

            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Some(input) = container.query_selector("#discount-input").ok().flatten() {
                    if let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                        let code = input.value().trim().to_string();
                        if code.is_empty() {
                            discount_code.set(None);
                        } else {
                            discount_code.set(Some(code));
                        }
                    }
                }
            }) as Box<dyn Fn(_)>);

            let _ = button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }

    fn setup_clear_button(
        container: &Element,
        cart: &Signal<Vec<CartItem>>,
        discount_code: &Signal<Option<String>>,
    ) {
        if let Some(button) = container.query_selector("#clear-cart").ok().flatten() {
            let cart = cart.clone();
            let discount_code = discount_code.clone();

            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                ferric_core::reactive::batch(|| {
                    cart.set(vec![]);
                    discount_code.set(None);
                });
            }) as Box<dyn Fn(_)>);

            let _ = button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }
}

fn inject_styles(id: &str, css: &str) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if document.get_element_by_id(id).is_some() {
            return;
        }
        if let Ok(style) = document.create_element("style") {
            let _ = style.set_attribute("id", id);
            style.set_text_content(Some(css));
            if let Some(head) = document.head() {
                let _ = head.append_child(&style);
            }
        }
    }
}
