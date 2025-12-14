# Ferric Animations

Angular-inspired animation system for the Ferric framework.

## Features

- **🎭 Animation Triggers**: Named animation definitions with states and transitions
- **🎨 States**: Define CSS styles for different animation states
- **⚡ Transitions**: Smooth animations between states with timing functions
- **🎬 Keyframes**: Complex multi-step animations
- **📊 Stagger**: Animate lists with progressive delays
- **🛣️ Route Animations**: Transitions triggered by route changes
- **🌊 Web Animations API**: Leverages native browser animation capabilities

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ferric-animations = { path = "../ferric-animations" }
```

## Quick Start

### Basic Fade Animation

```rust
use ferric_animations::prelude::*;

let fade = trigger("fade", vec![
    state("void", vec![
        ("opacity", "0"),
    ]),
    state("*", vec![
        ("opacity", "1"),
    ]),
    transition("void => *", &animate("300ms ease-in")),
    transition("* => void", &animate("300ms ease-out")),
]);
```

### Slide Animation

```rust
use ferric_animations::prelude::*;

let slide = trigger("slideIn", vec![
    state("void", vec![
        ("transform", "translateX(-100%)"),
        ("opacity", "0"),
    ]),
    state("*", vec![
        ("transform", "translateX(0)"),
        ("opacity", "1"),
    ]),
    transition("void => *", &animate("400ms cubic-bezier(0.35, 0, 0.25, 1)")),
]);
```

### Using the Builder API

```rust
use ferric_animations::AnimationBuilder;

let trigger = AnimationBuilder::new("bounce")
    .state("start", vec![
        ("transform", "scale(1)"),
    ])
    .state("end", vec![
        ("transform", "scale(1.2)"),
    ])
    .transition("start => end", "200ms ease-out")
    .transition("end => start", "200ms ease-in")
    .build();
```

## Advanced Features

### Keyframe Animations

```rust
use ferric_animations::prelude::*;

let pulse = keyframes("pulse", vec![
    (0.0, vec![("transform", "scale(1)"), ("opacity", "1")]),
    (0.5, vec![("transform", "scale(1.1)"), ("opacity", "0.8")]),
    (1.0, vec![("transform", "scale(1)"), ("opacity", "1")]),
]);
```

### Stagger Animations

Animate lists with progressive delays:

```rust
use ferric_animations::prelude::*;

let stagger_config = StaggerBuilder::new(
    300,  // Duration for each item
    50    // Delay between items
)
.max_items(20)
.reverse()
.build();

// Apply to each item in a list
for (index, item) in items.iter().enumerate() {
    let timing = stagger_config.timing_for_index(index, items.len());
    // Animate with timing
}
```

### Route Animations

Trigger animations on route changes:

```rust
use ferric_animations::prelude::*;

let page_slide = route_animation(
    slide_trigger,
    "/dashboard/*",
    RouteAnimationType::Enter
);
```

## Animation States

### Special States

- **`void`**: Element entering or leaving the DOM
- **`*`**: Wildcard state (matches any state)

### Custom States

Define any named state:

```rust
state("active", vec![
    ("background-color", "#4CAF50"),
    ("color", "white"),
])
```

## Transition Patterns

- **Unidirectional**: `"state1 => state2"`
- **Bidirectional**: `"state1 <=> state2"`
- **From any**: `"* => state"`
- **To any**: `"state => *"`
- **Increment**: `":increment"` (numeric state increase)
- **Decrement**: `":decrement"` (numeric state decrease)
- **Any**: `"*"` (all transitions)

## Timing Functions

### Predefined Easings

- `linear`
- `ease`
- `ease-in`
- `ease-out`
- `ease-in-out`

### Custom Cubic Bezier

```rust
"300ms cubic-bezier(0.42, 0, 0.58, 1)"
```

### With Delay

```rust
"300ms 100ms ease-in"  // 300ms duration, 100ms delay
```

## Using Animations in Components

```rust
use ferric_macros::component;
use ferric_animations::prelude::*;

#[component(
    selector = "my-component",
    // animations = [fade_trigger]  // Future: component-level animations
)]
pub struct MyComponent {
    // Component implementation
}
```

## Web Animations API Integration

The animation player uses the native Web Animations API for smooth, performant animations:

```rust
use ferric_animations::AnimationPlayer;
use std::rc::Rc;

// Create player
let element = /* get DOM element */;
let trigger = Rc::new(fade_trigger);
let mut player = AnimationPlayer::new(element, trigger);

// Transition to new state
player.transition_to("visible").expect("animation failed");
```

## Performance Tips

1. **Use CSS Transforms**: Prefer `transform` and `opacity` for best performance
2. **Hardware Acceleration**: Transforms trigger GPU acceleration
3. **Batch Updates**: Use `batch()` with signals for multiple animations
4. **Stagger Wisely**: Limit max items for large lists

## Browser Support

Requires browsers with Web Animations API support:

- Chrome 36+
- Firefox 48+
- Safari 13.1+
- Edge 79+

For older browsers, consider using a polyfill.

## Examples

See `examples/animations-demo` for comprehensive examples including:

- Basic fade/slide animations
- Keyframe animations
- Stagger animations
- Route transitions
- Custom easing functions

## License

Licensed under the Apache License, Version 2.0.

---

Built with 🦀 Rust and ❤️ for the Ferric framework

