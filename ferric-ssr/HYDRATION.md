# Ferric Hydration

Comprehensive guide to server-side rendering (SSR) and client-side hydration in Ferric.

## Table of Contents

- [Overview](#overview)
- [Full Hydration](#full-hydration)
- [Partial Hydration (Islands Architecture)](#partial-hydration-islands-architecture)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Overview

Hydration is the process of attaching interactivity to server-rendered HTML. Ferric supports two hydration strategies:

1. **Full Hydration**: Hydrates the entire application
2. **Partial Hydration (Islands)**: Selectively hydrates interactive components

### Why Hydration?

- **Better SEO**: Search engines can crawl fully-rendered HTML
- **Faster First Paint**: Users see content immediately
- **Improved Performance**: Islands reduce JavaScript bundle size
- **Progressive Enhancement**: Static content works without JavaScript

## Full Hydration

Full hydration attaches interactivity to all components in your application.

### Server-Side Setup

```rust
use ferric_ssr::prelude::*;

// 1. Create a hydration strategy
let strategy = FullHydration::new()
    .with_root_id("app")
    .with_debug(); // Enable debug logging

// 2. Configure the HTML shell
let shell_config = ShellConfig::new()
    .with_title("My App")
    .with_root_id("app")
    .with_hydration(&strategy); // Add hydration script

// 3. Render your component
let (html, state) = render_to_string_with_state(&my_component)?;

// 4. Wrap in shell with state and hydration
let full_html = wrap_in_shell(&html, Some(&state), &shell_config);
```

### Client-Side Setup

```rust
use ferric::prelude::*;
use ferric::hydration::*;

#[wasm_bindgen(start)]
pub fn main() {
    // Initialize the global hydration API
    init_hydration_api().expect("Failed to initialize hydration");

    // Your components will now be hydrated automatically
    // when the page loads
}
```

### Generated HTML

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <title>My App</title>
</head>
<body>
    <div id="app">
        <!-- Your server-rendered content -->
        <div data-ferric-id="my-component" data-ferric-hydrate="true">
            <h1>Hello, World!</h1>
            <button>Click me</button>
        </div>
    </div>

    <!-- Serialized state -->
    <script id="__FERRIC_STATE__" type="application/json">
    {"my-component": {"count": 0}}
    </script>

    <!-- Hydration script -->
    <script type="module">
    // Ferric hydration logic
    // Attaches event listeners, restores state
    </script>

    <!-- Your WASM bundle -->
    <script type="module" src="/app.js"></script>
</body>
</html>
```

## Partial Hydration (Islands Architecture)

Islands architecture lets you mark specific components as "islands" of interactivity in a sea of static HTML.

### Benefits

- **Smaller Bundles**: Only interactive components load JavaScript
- **Faster TTI**: Time to Interactive is reduced
- **Better Performance**: Especially on slow devices/networks
- **Flexibility**: Choose when each island hydrates

### Server-Side Setup

```rust
use ferric_ssr::prelude::*;

// 1. Define your islands
let strategy = PartialHydration::new()
    // High-priority: Hydrate immediately
    .add_island(
        Island::new("header-nav")
            .with_priority(IslandPriority::High)
            .with_loading(IslandLoading::Eager)
    )
    // Normal priority: Hydrate when visible
    .add_island(
        Island::new("product-gallery")
            .with_priority(IslandPriority::Normal)
            .with_loading(IslandLoading::Visible)
    )
    // Low priority: Hydrate on interaction
    .add_island(
        Island::new("comment-form")
            .with_priority(IslandPriority::Low)
            .with_loading(IslandLoading::Interaction)
    )
    // Hydrate only on desktop
    .add_island(
        Island::new("desktop-sidebar")
            .with_loading(IslandLoading::Media)
            .with_media_query("(min-width: 1024px)")
    );

// 2. Configure shell with islands
let shell_config = ShellConfig::new()
    .with_title("E-commerce Page")
    .with_root_id("app")
    .with_hydration(&strategy);

// 3. Render with markers
let (html, state) = render_with_hydration(
    &my_component,
    &strategy,
    "product-gallery"
)?;
```

### Island Loading Strategies

| Strategy | When it Hydrates | Use Case |
|----------|------------------|----------|
| **Eager** | Immediately on page load | Critical UI (navigation, search) |
| **Visible** | When scrolled into view | Below-the-fold content |
| **Interaction** | On user hover/click/focus | Forms, modals, dropdowns |
| **Idle** | When browser is idle | Analytics, non-critical features |
| **Media** | Based on media query | Responsive components |

### Island Priority

Priority determines the order when multiple islands hydrate:

```rust
pub enum IslandPriority {
    Critical,  // Hydrate first
    High,      // Hydrate early
    Normal,    // Standard priority
    Low,       // Hydrate later
    None,      // Static (never hydrate)
}
```

## API Reference

### HydrationStrategy Trait

```rust
pub trait HydrationStrategy {
    fn strategy_type(&self) -> &str;
    fn generate_markers(&self, component_id: &str) -> HydrationMarkers;
    fn generate_hydration_script(&self, config: &HydrationConfig) -> String;
    fn should_hydrate(&self, component_id: &str) -> bool;
}
```

### FullHydration

```rust
let strategy = FullHydration::new()
    .with_root_id("app")           // Root element ID
    .with_state_script_id("state") // State script ID
    .with_debug();                 // Enable debug logging
```

### PartialHydration

```rust
let strategy = PartialHydration::new()
    .add_island(Island::new("my-component"))
    .with_config(HydrationConfig {
        root_id: "app".to_string(),
        state_script_id: "__STATE__".to_string(),
        debug: true,
        timeout_ms: 10000,
    });
```

### Island

```rust
let island = Island::new("component-id")
    .with_priority(IslandPriority::High)
    .with_loading(IslandLoading::Visible)
    .with_media_query("(min-width: 768px)")
    .with_prop("count", serde_json::json!(42));
```

### Client-Side API

```rust
// Initialize the hydration API
init_hydration_api()?;

// Create a hydration context
let ctx = HydrationContext::from_root_with_state("app", "__FERRIC_STATE__")?;

// Find components to hydrate
let targets = ctx.find_hydration_targets()?;

// Manual hydration management
let mut manager = HydrationManager::new();
manager.hydrate_all(&ctx, |id, el| {
    // Factory function to create components
    Some(MyComponent::new())
})?;
```

## Examples

### Example 1: Blog Post with Interactive Comments

```rust
let strategy = PartialHydration::new()
    // Static header and content
    // Interactive comment section
    .add_island(
        Island::new("comment-section")
            .with_loading(IslandLoading::Visible)
    );
```

### Example 2: E-commerce Product Page

```rust
let strategy = PartialHydration::new()
    // Critical: Add to cart button
    .add_island(
        Island::new("add-to-cart")
            .with_priority(IslandPriority::Critical)
            .with_loading(IslandLoading::Eager)
    )
    // Product image gallery (loads when visible)
    .add_island(
        Island::new("product-gallery")
            .with_loading(IslandLoading::Visible)
    )
    // Reviews section (loads on interaction)
    .add_island(
        Island::new("reviews")
            .with_loading(IslandLoading::Interaction)
    );
```

### Example 3: Marketing Landing Page

```rust
let strategy = PartialHydration::new()
    // Hero CTA button
    .add_island(
        Island::new("hero-cta")
            .with_priority(IslandPriority::High)
            .with_loading(IslandLoading::Eager)
    )
    // Email signup form (loads when visible)
    .add_island(
        Island::new("newsletter-form")
            .with_loading(IslandLoading::Visible)
    )
    // Live chat widget (loads when idle)
    .add_island(
        Island::new("live-chat")
            .with_priority(IslandPriority::Low)
            .with_loading(IslandLoading::Idle)
    );
```

## Best Practices

### 1. Choose the Right Strategy

**Use Full Hydration when:**
- Building a single-page application (SPA)
- Most/all content is interactive
- SEO is the primary concern
- Bundle size is not a constraint

**Use Partial Hydration when:**
- Building content-heavy sites
- Only parts of the page are interactive
- Performance is critical
- Targeting low-end devices/slow networks

### 2. Island Design

**Good Island Candidates:**
- Forms and inputs
- Interactive widgets (carousels, accordions)
- Real-time updates (live chat, notifications)
- User interactions (likes, comments, shares)

**Keep Static:**
- Navigation (if purely links)
- Footer content
- Article text
- Static images and media
- Copyright/legal text

### 3. Loading Strategy Guidelines

```rust
// Critical UI - loads immediately
IslandLoading::Eager + IslandPriority::Critical

// Above the fold interactivity
IslandLoading::Eager + IslandPriority::High

// Below the fold
IslandLoading::Visible + IslandPriority::Normal

// Forms, modals
IslandLoading::Interaction + IslandPriority::Normal

// Analytics, chat widgets
IslandLoading::Idle + IslandPriority::Low
```

### 4. State Management

```rust
// Add state for hydration
let mut state_builder = StateBuilder::new();
state_builder = state_builder
    .add("my-component", &component_state)?;

let state = state_builder.build()?;
```

### 5. Testing Hydration

```bash
# Test with JavaScript disabled
# Your static content should still work!

# Test hydration timing
# Open DevTools > Network tab
# Watch when islands load
```

### 6. Debugging

```rust
// Enable debug mode
let strategy = FullHydration::new()
    .with_debug();

// Check browser console for:
// [Ferric Hydration] Starting full hydration...
// [Ferric Hydration] Found 5 components to hydrate
// [Ferric Islands] Hydrating island: my-component priority: high
```

### 7. Performance Monitoring

```javascript
// Listen for hydration events
window.addEventListener('ferric:hydrated', (e) => {
    console.log('Hydration complete:', e.detail);
});

window.addEventListener('ferric:island-hydrated', (e) => {
    console.log('Island hydrated:', e.detail.islandId);
});
```

## Troubleshooting

### Components Not Hydrating

1. Check that `init_hydration_api()` is called
2. Verify `data-ferric-hydrate="true"` attribute
3. Check `data-ferric-id` matches island configuration
4. Look for errors in browser console

### State Not Restoring

1. Verify state script tag is present
2. Check state script ID matches config
3. Ensure state is valid JSON
4. Check serialization/deserialization

### Performance Issues

1. Too many eager islands? Use `Visible` or `Interaction`
2. Bundle size too large? Split into more islands
3. Hydration timeout? Increase `timeout_ms`
4. Use Chrome DevTools Performance tab

## Further Reading

- [Islands Architecture](https://jasonformat.com/islands-architecture/)
- [Progressive Enhancement](https://developer.mozilla.org/en-US/docs/Glossary/Progressive_Enhancement)
- [SSR Best Practices](https://web.dev/rendering-on-the-web/)

---

For more examples, see `examples/ssr-demo/src/hydration_example.rs`.

