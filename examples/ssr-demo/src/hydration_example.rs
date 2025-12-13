//! Example demonstrating full and partial hydration strategies.

use ferric_ssr::prelude::*;
use serde::{Deserialize, Serialize};

/// Example component state that will be serialized and hydrated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterState {
    pub count: i32,
    pub label: String,
}

/// Example showing full hydration.
pub fn full_hydration_example() -> String {
    // Create hydration strategy
    let strategy = FullHydration::new()
        .with_root_id("app")
        .with_debug();

    // Configure shell with hydration
    let shell_config = ShellConfig::new()
        .with_title("Full Hydration Example")
        .with_root_id("app")
        .with_hydration(&strategy)
        .with_head_extra(r#"<style>
            body { font-family: sans-serif; padding: 20px; }
            .counter { border: 1px solid #ccc; padding: 20px; margin: 10px 0; }
            button { padding: 10px 20px; font-size: 16px; cursor: pointer; }
        </style>"#);

    // Simulate rendering a component
    let component_html = r#"<div class="counter">
        <h2>Counter Component</h2>
        <p>Count: <span id="count">0</span></p>
        <button onclick="increment()">Increment</button>
    </div>"#;

    // Create state
    let mut state_builder = StateBuilder::new();
    state_builder = state_builder
        .add("counter", &CounterState {
            count: 0,
            label: "Counter".to_string(),
        })
        .unwrap();
    let state = state_builder.build().unwrap();

    // Wrap in shell with hydration
    wrap_in_shell(component_html, Some(&state), &shell_config)
}

/// Example showing partial hydration (islands).
pub fn partial_hydration_example() -> String {
    // Create islands configuration
    let strategy = PartialHydration::new()
        .add_island(
            Island::new("interactive-counter")
                .with_priority(IslandPriority::High)
                .with_loading(IslandLoading::Eager)
        )
        .add_island(
            Island::new("lazy-comments")
                .with_priority(IslandPriority::Low)
                .with_loading(IslandLoading::Visible)
        )
        .add_island(
            Island::new("interactive-form")
                .with_priority(IslandPriority::Normal)
                .with_loading(IslandLoading::Interaction)
        );

    // Configure shell with island hydration
    let shell_config = ShellConfig::new()
        .with_title("Partial Hydration (Islands) Example")
        .with_root_id("app")
        .with_hydration(&strategy)
        .with_head_extra(r#"<style>
            body { font-family: sans-serif; padding: 20px; max-width: 800px; margin: 0 auto; }
            .island { border: 2px solid #4CAF50; padding: 20px; margin: 20px 0; border-radius: 8px; }
            .static { border: 2px solid #ccc; padding: 20px; margin: 20px 0; border-radius: 8px; }
            button { padding: 10px 20px; font-size: 16px; cursor: pointer; }
            .priority-badge {
                display: inline-block;
                padding: 4px 8px;
                background: #4CAF50;
                color: white;
                border-radius: 4px;
                font-size: 12px;
                margin-left: 10px;
            }
            [data-ferric-hydrated="true"] {
                box-shadow: 0 0 10px rgba(76, 175, 80, 0.3);
            }
        </style>"#);

    // Build page with islands and static content
    let page_html = r#"
        <!-- Static header - no hydration needed -->
        <div class="static">
            <h1>Islands Architecture Demo</h1>
            <p>This page demonstrates selective hydration. Green-bordered sections are interactive "islands"
            that will be hydrated, while gray sections remain static HTML.</p>
        </div>

        <!-- High priority island - hydrates immediately -->
        <div class="island" data-ferric-id="interactive-counter" data-ferric-hydrate="true" data-ferric-priority="high">
            <h2>Interactive Counter <span class="priority-badge">HIGH PRIORITY</span></h2>
            <p>This island hydrates immediately on page load.</p>
            <p>Count: <span id="count">0</span></p>
            <button>Increment</button>
        </div>

        <!-- Static content -->
        <div class="static">
            <h2>Static Content</h2>
            <p>This section is just HTML - no JavaScript, no hydration. Perfect for content that doesn't need interactivity!</p>
            <ul>
                <li>Faster initial load</li>
                <li>Less JavaScript to download</li>
                <li>Better performance on low-end devices</li>
            </ul>
        </div>

        <!-- Normal priority island - hydrates on interaction -->
        <div class="island" data-ferric-id="interactive-form" data-ferric-hydrate="true" data-ferric-priority="normal">
            <h2>Interactive Form <span class="priority-badge">NORMAL PRIORITY</span></h2>
            <p>This island hydrates when you interact with it (hover, click, focus).</p>
            <form>
                <input type="text" placeholder="Your name" />
                <button type="submit">Submit</button>
            </form>
        </div>

        <!-- More static content -->
        <div class="static">
            <h2>More Static Content</h2>
            <p>Another section that doesn't need JavaScript. The server rendered this HTML, and it stays that way.</p>
        </div>

        <!-- Low priority island - hydrates when visible -->
        <div class="island" data-ferric-id="lazy-comments" data-ferric-hydrate="true" data-ferric-priority="low">
            <h2>Comments Section <span class="priority-badge">LOW PRIORITY</span></h2>
            <p>This island hydrates when it scrolls into view (lazy loading).</p>
            <div class="comment">
                <strong>User1:</strong> Great article!
            </div>
            <div class="comment">
                <strong>User2:</strong> Thanks for sharing!
            </div>
            <button>Load More Comments</button>
        </div>

        <!-- Footer - static -->
        <div class="static">
            <hr>
            <p><small>© 2024 Ferric Framework. Built with Islands Architecture.</small></p>
        </div>
    "#;

    // Create state for islands
    let mut state_builder = StateBuilder::new();
    state_builder = state_builder
        .add("interactive-counter", &CounterState {
            count: 0,
            label: "Counter".to_string(),
        })
        .unwrap()
        .add("interactive-form", &serde_json::json!({
            "submitted": false
        }))
        .unwrap();
    let state = state_builder.build().unwrap();

    // Wrap in shell with hydration
    wrap_in_shell(page_html, Some(&state), &shell_config)
}

/// Example comparing hydration strategies.
pub fn comparison_example() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hydration Strategies Comparison</title>
    <style>
        body {{ font-family: sans-serif; padding: 20px; max-width: 1200px; margin: 0 auto; }}
        .comparison {{ display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }}
        .strategy {{ border: 2px solid #333; padding: 20px; border-radius: 8px; }}
        h2 {{ margin-top: 0; }}
        .pros {{ color: green; }}
        .cons {{ color: red; }}
        code {{ background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }}
    </style>
</head>
<body>
    <h1>Ferric Hydration Strategies</h1>

    <div class="comparison">
        <div class="strategy">
            <h2>🌊 Full Hydration</h2>
            <p>Hydrates the entire application, attaching interactivity to all components.</p>

            <h3 class="pros">✓ Pros:</h3>
            <ul>
                <li>Simple to implement</li>
                <li>All components are interactive immediately</li>
                <li>Consistent behavior across the app</li>
            </ul>

            <h3 class="cons">✗ Cons:</h3>
            <ul>
                <li>Larger JavaScript bundle</li>
                <li>Longer time to interactive (TTI)</li>
                <li>Hydrates static content unnecessarily</li>
            </ul>

            <h3>Best for:</h3>
            <ul>
                <li>Highly interactive applications</li>
                <li>Single-page applications (SPAs)</li>
                <li>When most content needs interactivity</li>
            </ul>

            <h3>Example:</h3>
            <pre><code>let strategy = FullHydration::new()
    .with_root_id("app")
    .with_debug();

let config = ShellConfig::new()
    .with_hydration(&strategy);</code></pre>
        </div>

        <div class="strategy">
            <h2>🏝️ Partial Hydration (Islands)</h2>
            <p>Selectively hydrates interactive "islands" while leaving static content as HTML.</p>

            <h3 class="pros">✓ Pros:</h3>
            <ul>
                <li>Smaller JavaScript bundles</li>
                <li>Faster time to interactive (TTI)</li>
                <li>Better performance on slow devices</li>
                <li>Progressive enhancement</li>
            </ul>

            <h3 class="cons">✗ Cons:</h3>
            <ul>
                <li>More complex setup</li>
                <li>Need to identify which components are islands</li>
                <li>State management between islands</li>
            </ul>

            <h3>Best for:</h3>
            <ul>
                <li>Content-heavy sites</li>
                <li>Marketing pages with interactive widgets</li>
                <li>Documentation sites</li>
                <li>E-commerce product pages</li>
            </ul>

            <h3>Example:</h3>
            <pre><code>let strategy = PartialHydration::new()
    .add_island(
        Island::new("interactive-button")
            .with_priority(IslandPriority::High)
            .with_loading(IslandLoading::Visible)
    );

let config = ShellConfig::new()
    .with_hydration(&strategy);</code></pre>
        </div>
    </div>

    <h2>Loading Strategies</h2>
    <table border="1" cellpadding="10" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Strategy</th>
            <th>Description</th>
            <th>Use Case</th>
        </tr>
        <tr>
            <td><code>Eager</code></td>
            <td>Hydrate immediately on page load</td>
            <td>Critical interactive components above the fold</td>
        </tr>
        <tr>
            <td><code>Visible</code></td>
            <td>Hydrate when element is visible in viewport</td>
            <td>Below-the-fold components, lazy loading</td>
        </tr>
        <tr>
            <td><code>Interaction</code></td>
            <td>Hydrate on user interaction (click, hover, focus)</td>
            <td>Forms, modals, dropdowns</td>
        </tr>
        <tr>
            <td><code>Idle</code></td>
            <td>Hydrate when browser is idle</td>
            <td>Low-priority features, analytics</td>
        </tr>
        <tr>
            <td><code>Media</code></td>
            <td>Hydrate based on media query</td>
            <td>Responsive components (desktop vs mobile)</td>
        </tr>
    </table>
</body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_hydration_example() {
        let html = full_hydration_example();
        assert!(html.contains("Full Hydration Example"));
        assert!(html.contains("ferric:hydrated"));
        assert!(html.contains("__FERRIC_STATE__"));
    }

    #[test]
    fn test_partial_hydration_example() {
        let html = partial_hydration_example();
        assert!(html.contains("Islands Architecture"));
        assert!(html.contains("data-ferric-id"));
        assert!(html.contains("data-ferric-hydrate"));
    }

    #[test]
    fn test_comparison_example() {
        let html = comparison_example();
        assert!(html.contains("Full Hydration"));
        assert!(html.contains("Partial Hydration"));
        assert!(html.contains("Islands"));
    }
}

