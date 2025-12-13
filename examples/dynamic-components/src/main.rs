//! Example demonstrating dynamic component creation.

use ferric_core::component::{
    ComponentFactory, ComponentMetadata, register_component,
    create_component, create_components,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Register some components
    register_component("app-button", ComponentMetadata {
        selector: "app-button".to_string(),
        template: Some("<button>Click me!</button>".to_string()),
        styles: vec!["button { padding: 10px; color: blue; }".to_string()],
        ..Default::default()
    });

    register_component("app-card", ComponentMetadata {
        selector: "app-card".to_string(),
        template: Some(r#"
            <div class="card">
                <h3>Card Title</h3>
                <p>Card content goes here</p>
            </div>
        "#.to_string()),
        styles: vec![
            ".card { border: 1px solid #ccc; padding: 20px; border-radius: 8px; }".to_string(),
        ],
        ..Default::default()
    });

    // Get the container element
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");
    let container = document
        .get_element_by_id("app")
        .expect("no #app element");

    // Example 1: Create a single component
    web_sys::console::log_1(&"Creating single component...".into());
    let button = create_component("app-button", Some(&container))?;
    web_sys::console::log_1(&format!("Created button with instance ID: {}", button.instance_id()).into());

    // Example 2: Create multiple components
    web_sys::console::log_1(&"Creating multiple components...".into());
    let components = create_components(
        &["app-card", "app-card", "app-button"],
        Some(&container),
    )?;
    web_sys::console::log_1(&format!("Created {} components", components.len()).into());

    // Example 3: Using a custom factory
    web_sys::console::log_1(&"Using custom factory...".into());
    let factory = ComponentFactory::new();

    let card1 = factory.create("app-card", Some(&container))?;
    let card2 = factory.create("app-card", Some(&container))?;

    web_sys::console::log_1(&format!(
        "Factory has {} active instances",
        factory.instance_count()
    ).into());

    // Example 4: Destroying components
    web_sys::console::log_1(&"Destroying a component...".into());
    factory.destroy(card1.instance_id())?;
    web_sys::console::log_1(&format!(
        "Factory now has {} active instances",
        factory.instance_count()
    ).into());

    web_sys::console::log_1(&"Dynamic components example complete!".into());
    Ok(())
}

