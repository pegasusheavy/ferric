//! Counter component with state serialization.

use ferric_ssr::prelude::*;
use serde::{Deserialize, Serialize};

/// Counter state that will be serialized for hydration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterState {
    pub value: i32,
    pub step: i32,
}

/// Counter component demonstrating state serialization.
pub struct Counter {
    state: CounterState,
}

impl Counter {
    /// Create a new counter with an initial value.
    pub fn new(initial_value: i32) -> Self {
        Self {
            state: CounterState {
                value: initial_value,
                step: 1,
            },
        }
    }

    /// Create a counter with custom step.
    pub fn with_step(initial_value: i32, step: i32) -> Self {
        Self {
            state: CounterState {
                value: initial_value,
                step,
            },
        }
    }
}

impl Renderable for Counter {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        // Add state for hydration
        ctx.add_state("counter", &self.state)?;

        let mut html = HtmlRenderer::new();

        html.open_tag("div")
            .attr("class", "counter")
            .attr("data-component", "counter")
            .close_open();

        html.element("h1", &[], "Interactive Counter");
        html.element(
            "p",
            &[],
            "This counter's state is serialized for client-side hydration.",
        );

        html.open_tag("div")
            .attr("class", "counter-display")
            .attr("style", "text-align: center; padding: 2rem;")
            .close_open();

        html.open_tag("div")
            .attr("class", "counter-value")
            .attr(
                "style",
                "font-size: 5rem; font-weight: bold; color: #667eea; margin: 1rem 0;",
            )
            .close_open();
        html.text(&self.state.value.to_string());
        html.close_tag("div");

        html.open_tag("div")
            .attr("class", "counter-controls")
            .attr("style", "display: flex; gap: 1rem; justify-content: center;")
            .close_open();

        // Decrement button
        html.open_tag("button")
            .attr("data-action", "decrement")
            .attr(
                "style",
                "padding: 0.75rem 2rem; font-size: 1.5rem; border: none; \
                 border-radius: 8px; background: #e74c3c; color: white; cursor: pointer;",
            )
            .close_open();
        html.text("−");
        html.close_tag("button");

        // Increment button
        html.open_tag("button")
            .attr("data-action", "increment")
            .attr(
                "style",
                "padding: 0.75rem 2rem; font-size: 1.5rem; border: none; \
                 border-radius: 8px; background: #27ae60; color: white; cursor: pointer;",
            )
            .close_open();
        html.text("+");
        html.close_tag("button");

        html.close_tag("div");
        html.close_tag("div");

        // Step info
        html.open_tag("div")
            .attr("style", "text-align: center; color: #888; margin-top: 1rem;")
            .close_open();
        html.text(&format!("Step: {}", self.state.step));
        html.close_tag("div");

        // Platform info
        html.open_tag("div")
            .attr(
                "style",
                "margin-top: 2rem; padding: 1rem; background: #f8f9fa; border-radius: 8px;",
            )
            .close_open();
        html.element("h3", &[], "Platform Detection");

        let platform = if is_server() {
            "🖥️ Server (this HTML was rendered on the server)"
        } else {
            "🌐 Browser"
        };
        html.element("p", &[], platform);

        html.close_tag("div");

        html.close_tag("div");

        ctx.exit();
        Ok(html.finish())
    }
}

