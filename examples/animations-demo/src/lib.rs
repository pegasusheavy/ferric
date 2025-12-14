//! Ferric Animations Demo
//!
//! Comprehensive examples of the animation system.

use ferric_animations::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, window};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set up panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"🎬 Ferric Animations Demo Loaded!".into());

    setup_examples()?;

    Ok(())
}

fn setup_examples() -> Result<(), JsValue> {
    let window = window().expect("no global window");
    let document = window.document().expect("no document");

    // Example 1: Fade Animation
    setup_fade_example(&document)?;

    // Example 2: Slide Animation
    setup_slide_example(&document)?;

    // Example 3: Bounce Animation
    setup_bounce_example(&document)?;

    // Example 4: Stagger Animation
    setup_stagger_example(&document)?;

    Ok(())
}

fn setup_fade_example(document: &Document) -> Result<(), JsValue> {
    web_sys::console::log_1(&"Setting up fade example".into());

    // Create fade trigger
    let fade = trigger("fade", vec![
        state("hidden", vec![("opacity", "0")]),
        state("visible", vec![("opacity", "1")]),
        transition("hidden => visible", "600ms ease-in"),
        transition("visible => hidden", "600ms ease-out"),
    ]);

    // Log the trigger
    web_sys::console::log_1(&format!("Created fade trigger: {:?}", fade).into());

    Ok(())
}

fn setup_slide_example(document: &Document) -> Result<(), JsValue> {
    web_sys::console::log_1(&"Setting up slide example".into());

    // Create slide trigger with builder
    let slide = AnimationBuilder::new("slide")
        .state("left", vec![
            ("transform", "translateX(-100%)"),
            ("opacity", "0"),
        ])
        .state("center", vec![
            ("transform", "translateX(0)"),
            ("opacity", "1"),
        ])
        .state("right", vec![
            ("transform", "translateX(100%)"),
            ("opacity", "0"),
        ])
        .transition("left => center", "400ms cubic-bezier(0.35, 0, 0.25, 1)")
        .transition("center => right", "400ms cubic-bezier(0.35, 0, 0.25, 1)")
        .build();

    web_sys::console::log_1(&format!("Created slide trigger: {:?}", slide).into());

    Ok(())
}

fn setup_bounce_example(document: &Document) -> Result<(), JsValue> {
    web_sys::console::log_1(&"Setting up bounce example".into());

    // Create bounce keyframes
    let bounce = keyframes("bounce", vec![
        (0.0, vec![("transform", "translateY(0)")]),
        (0.2, vec![("transform", "translateY(-30px)")]),
        (0.4, vec![("transform", "translateY(0)")]),
        (0.5, vec![("transform", "translateY(-15px)")]),
        (0.6, vec![("transform", "translateY(0)")]),
        (1.0, vec![("transform", "translateY(0)")]),
    ]);

    web_sys::console::log_1(&format!("Created bounce keyframes: {:?}", bounce).into());

    // Show CSS output
    let css = bounce.to_css();
    web_sys::console::log_1(&format!("Bounce CSS:\n{}", css).into());

    Ok(())
}

fn setup_stagger_example(document: &Document) -> Result<(), JsValue> {
    web_sys::console::log_1(&"Setting up stagger example".into());

    // Create stagger configuration
    let config = StaggerBuilder::new(300, 50)
        .easing(EasingFunction::EaseOut)
        .max_items(10)
        .build();

    web_sys::console::log_1(&format!("Created stagger config: {:?}", config).into());

    // Demonstrate timing for different items
    for i in 0..5 {
        let delay = config.delay_for_index(i, 10);
        web_sys::console::log_1(&format!("Item {} delay: {}ms", i, delay).into());
    }

    Ok(())
}

// Example animations that can be used in components
pub fn example_animations() -> Vec<AnimationTrigger> {
    vec![
        // Fade in/out
        trigger("fadeInOut", vec![
            state("void", vec![("opacity", "0")]),
            state("*", vec![("opacity", "1")]),
            transition("void => *", "300ms ease-in"),
            transition("* => void", "300ms ease-out"),
        ]),
        
        // Expand/collapse
        trigger("expandCollapse", vec![
            state("collapsed", vec![
                ("height", "0"),
                ("overflow", "hidden"),
            ]),
            state("expanded", vec![
                ("height", "*"),
                ("overflow", "visible"),
            ]),
            transition("collapsed <=> expanded", "400ms ease-in-out"),
        ]),
        
        // Rotate
        trigger("rotate", vec![
            state("default", vec![("transform", "rotate(0deg)")]),
            state("rotated", vec![("transform", "rotate(180deg)")]),
            transition("default <=> rotated", "300ms ease-in-out"),
        ]),
        
        // Scale
        trigger("scale", vec![
            state("small", vec![("transform", "scale(0.8)")]),
            state("normal", vec![("transform", "scale(1)")]),
            state("large", vec![("transform", "scale(1.2)")]),
            transition("* => *", "200ms ease-out"),
        ]),
    ]
}

#[wasm_bindgen]
pub fn log_example_animations() {
    let animations = example_animations();
    web_sys::console::log_1(&format!("📦 Loaded {} example animations", animations.len()).into());
    
    for animation in animations {
        web_sys::console::log_1(&format!(
            "  - {} ({} states, {} transitions)",
            animation.name,
            animation.states.len(),
            animation.transitions.len()
        ).into());
    }
}

