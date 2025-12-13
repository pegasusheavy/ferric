# Ferric SSR Demo

This example demonstrates the server-side rendering capabilities of the Ferric framework's `ferric-ssr` module.

## Features Demonstrated

- **Component Rendering**: Create components implementing the `Renderable` trait
- **HTML Generation**: Use `HtmlRenderer` for safe, structured HTML output
- **State Serialization**: Serialize component state for client-side hydration
- **Platform Detection**: Use `is_server()` and `is_browser()` for isomorphic code
- **HTTP Server**: Built-in Hyper-based HTTP server with routing
- **JSON API**: Return JSON responses alongside HTML pages

## Running the Example

```bash
# From the workspace root
cargo run -p ssr-demo

# Or from the example directory
cd examples/ssr-demo
cargo run
```

Then open your browser to [http://localhost:3000](http://localhost:3000)

## Available Routes

| Route | Description |
|-------|-------------|
| `GET /` | Home page with feature overview |
| `GET /about` | About page explaining SSR concepts |
| `GET /counter` | Counter with state serialization for hydration |
| `GET /todos` | Todo list example |
| `GET /user` | User profile card |
| `GET /api/data` | JSON API endpoint |

## Project Structure

```
ssr-demo/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs              # Server setup and route handlers
    └── components/
        ├── mod.rs           # Module exports
        ├── layout.rs        # Main layout wrapper
        ├── pages.rs         # Page components (Home, About, NotFound)
        ├── counter.rs       # Counter with state
        ├── todo.rs          # Todo list components
        └── user.rs          # User profile card
```

## Creating Renderable Components

```rust
use ferric_ssr::prelude::*;

struct MyComponent {
    message: String,
}

impl Renderable for MyComponent {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;  // Track rendering depth

        let mut html = HtmlRenderer::new();
        html.open_tag("div")
            .attr("class", "my-component")
            .close_open();
        html.element("h1", &[], &self.message);
        html.close_tag("div");

        ctx.exit();
        Ok(html.finish())
    }
}
```

## State Serialization for Hydration

```rust
use ferric_ssr::{render_to_string_with_state, prelude::*};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CounterState {
    value: i32,
}

impl Renderable for Counter {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        // Add state to context for serialization
        ctx.add_state("counter", &self.state)?;

        // ... render HTML
    }
}

// In your handler:
let (html, state) = render_to_string_with_state(&counter)?;
// state.as_json() gives you the serialized state to embed in the page
```

## Platform Detection

```rust
use ferric_ssr::prelude::*;

// Check current platform
if is_server() {
    println!("Running on server");
}

// Execute code only on server
let data = server_or(
    || expensive_server_computation(),
    || default_value()
);
```

## Building for Production

```bash
cargo build --release -p ssr-demo
./target/release/ssr-demo
```

## Learn More

- [Ferric Framework Documentation](https://github.com/username/ferric)
- [Hyper HTTP Library](https://hyper.rs/)
- [Tokio Async Runtime](https://tokio.rs/)

