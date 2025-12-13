# Ferric

An Angular-inspired frontend framework built with Rust and WebAssembly.

## Features

- **Components** - Reusable UI building blocks with encapsulated templates, styles, and logic
- **Reactive State** - Fine-grained reactivity with signals, computed values, and effects
- **Templates** - Declarative view definitions with Angular-like binding syntax
- **Dependency Injection** - Hierarchical service container for managing application services
- **Routing** - Client-side navigation with guards and route resolvers
- **Directives** - Extend element behavior with structural (`*if`, `*for`) and attribute directives
- **Lifecycle Hooks** - Component initialization, updates, and cleanup hooks

## Project Structure

```
ferric/
├── Cargo.toml              # Rust project manifest
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── utils.rs            # Utility functions
│   ├── component/          # Component system
│   │   ├── mod.rs
│   │   ├── context.rs      # Component runtime context
│   │   ├── metadata.rs     # Component decorators/metadata
│   │   └── registry.rs     # Global component registry
│   ├── reactive/           # Reactive state management
│   │   ├── mod.rs
│   │   ├── signal.rs       # Mutable reactive state
│   │   ├── computed.rs     # Derived reactive values
│   │   └── effect.rs       # Side effects
│   ├── di/                 # Dependency injection
│   │   ├── mod.rs
│   │   ├── injector.rs     # Hierarchical injector
│   │   ├── provider.rs     # Service providers
│   │   └── token.rs        # Injection tokens
│   ├── router/             # Client-side routing
│   │   ├── mod.rs
│   │   ├── config.rs       # Route configuration
│   │   ├── guard.rs        # Route guards
│   │   ├── outlet.rs       # Router outlet
│   │   └── service.rs      # Router service
│   ├── template/           # Template system
│   │   ├── mod.rs
│   │   ├── binding.rs      # Data binding types
│   │   ├── parser.rs       # Template parser
│   │   └── compiler.rs     # Template compiler
│   ├── directives/         # Built-in directives
│   │   ├── mod.rs
│   │   ├── structural.rs   # *if, *for, *switch
│   │   └── attribute.rs    # Class, style, etc.
│   ├── dom/                # DOM utilities
│   │   ├── mod.rs
│   │   ├── element.rs      # Element manipulation
│   │   ├── events.rs       # Event handling
│   │   └── query.rs        # DOM querying
│   └── lifecycle/          # Lifecycle hooks
│       └── mod.rs
└── examples/               # Example applications
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Using the CLI (Recommended)

The Ferric CLI provides the easiest way to create and develop Ferric applications:

```bash
# Install the CLI
cargo install --path ferric-cli

# Create a new project
ferric new my-app
cd my-app

# Start development server with live reload
ferric serve

# Generate a component (with external HTML and SCSS by default)
ferric generate component header

# Build for production
ferric build --release
```

See [ferric-cli/README.md](ferric-cli/README.md) for full CLI documentation.

### Manual Building

```bash
# Build the WASM package
wasm-pack build --target web

# Or for Node.js
wasm-pack build --target nodejs
```

### Usage

```rust
use ferric::*;

// Define a component
pub struct AppComponent {
    title: Signal<String>,
}

impl Component for AppComponent {
    fn selector(&self) -> &'static str {
        "app-root"
    }

    fn template(&self) -> &str {
        r#"
        <div class="app">
            <h1>{{ title }}</h1>
            <button (click)="on_click()">Click me</button>
        </div>
        "#
    }

    fn render(&self) -> Result<web_sys::Element, String> {
        // Rendering implementation
        todo!()
    }
}

impl Lifecycle for AppComponent {
    fn on_init(&mut self) {
        self.title.set("Hello, Ferric!".to_string());
    }
}
```

## Template Syntax

Ferric uses Angular-like template syntax:

| Syntax | Description | Example |
|--------|-------------|---------|
| `{{ }}` | Interpolation | `{{ name }}` |
| `[prop]` | Property binding | `[value]="count"` |
| `(event)` | Event binding | `(click)="handle()"` |
| `[(prop)]` | Two-way binding | `[(value)]="text"` |
| `[attr.x]` | Attribute binding | `[attr.aria-label]="label"` |
| `[class.x]` | Class binding | `[class.active]="isActive"` |
| `[style.x]` | Style binding | `[style.color]="textColor"` |
| `*if` | Conditional rendering | `*if="condition"` |
| `*for` | List rendering | `*for="item of items"` |
| `#ref` | Template reference | `#myInput` |

### Template Rendering

```rust
use ferric::template::*;

// Create a context with values
let context = TemplateContext::new();
context.set("name", "Ferric".to_string());
context.set("count", 42);
context.set("isActive", true);

// Compile and render a template
let template = r#"
    <div class="greeting" [class.active]="isActive">
        <h1>Hello, {{ name }}!</h1>
        <p>Count: {{ count }}</p>
        <button (click)="increment()">+</button>
    </div>
"#;

let compiled = compile_template(template)?;
let instance = compiled.render(context)?;

// Access the root DOM element
let root_element = instance.root();

// Update bindings when context changes
instance.update();
```

### External Template Files

Use `include_str!()` to load templates from external files at compile time:

```rust
// Component with external template and styles
const TEMPLATE: &str = include_str!("my_component.html");
const STYLES: &str = include_str!("my_component.css");
```

## Dependency Injection

Ferric provides a full-featured DI system with hierarchical injectors:

### Basic Service Registration

```rust
use ferric::di::*;

// Define a service
struct UserService {
    db: Rc<DatabaseService>,
}

impl Injectable for UserService {
    fn create(injector: &Injector) -> Self {
        Self {
            db: injector.resolve_required::<DatabaseService>(),
        }
    }
}

// Create injector and register
let injector = Injector::root();
injector.register_singleton::<UserService>();

let service = injector.resolve::<UserService>().unwrap();
```

### Injection Tokens

```rust
// Define tokens for non-class values
const API_URL: InjectionToken<String> = InjectionToken::with_id("API_URL", 1);
const MAX_CONNECTIONS: InjectionToken<u32> = InjectionToken::with_id("MAX_CONNECTIONS", 2);

injector.register_token(&API_URL, "https://api.example.com".to_string());
injector.register_token(&MAX_CONNECTIONS, 10);

let url = injector.resolve_token(&API_URL).unwrap();
```

### Multi-providers

```rust
// Multiple implementations for a single token
const VALIDATORS: MultiToken<Box<dyn Validator>> = MultiToken::with_id("VALIDATORS", 3);

injector.add_multi(&VALIDATORS, Box::new(RequiredValidator));
injector.add_multi(&VALIDATORS, Box::new(EmailValidator));

let all_validators = injector.resolve_multi(&VALIDATORS);
```

### Resolution Modifiers

```rust
// Optional - won't error if not found
let service = Optional::new(&injector).resolve::<MyService>();

// Self - only check current injector
let service = Self_::new(&injector).resolve::<MyService>();

// SkipSelf - skip current, check parent
let service = SkipSelf::new(&injector).resolve::<MyService>();

// Combined with fluent API
let service = Resolve::new(&injector)
    .optional()
    .skip_self()
    .get::<MyService>();
```

### Hierarchical Injectors

```rust
let parent = Rc::new(Injector::root());
parent.register_singleton::<DatabaseService>();

let child = Injector::child(Rc::clone(&parent));
child.register_singleton::<UserService>();

// Child resolves from parent automatically
let user_service = child.resolve::<UserService>().unwrap();
```

## Reactive System (Signals)

Ferric provides a complete reactive system with automatic dependency tracking:

### Signals

```rust
use ferric::reactive::*;

// Create a signal
let count = signal(0);

// Read the value
let value = count.get(); // Automatically tracks as dependency

// Update the value
count.set(10);
count.update(|n| n + 1);
count.mutate(|v| v.push(item)); // For collections
```

### Computed Values

```rust
let first = signal("John".to_string());
let last = signal("Doe".to_string());

// Computed automatically tracks dependencies
let full_name = computed(move || {
    format!("{} {}", first.get(), last.get())
});

assert_eq!(full_name.get(), "John Doe");
first.set("Jane".to_string());
assert_eq!(full_name.get(), "Jane Doe"); // Auto-updated!
```

### Effects

```rust
let count = signal(0);

// Effect runs immediately and when dependencies change
let fx = effect(move || {
    console::log_1(&format!("Count: {}", count.get()).into());
});

count.set(1); // Logs "Count: 1"
count.set(2); // Logs "Count: 2"

fx.stop(); // Stop the effect
```

### Batching

```rust
let a = signal(1);
let b = signal(2);

// Updates are batched - subscribers notified once at the end
batch(|| {
    a.set(10);
    b.set(20);
});
```

### Resources (Async Data)

```rust
let user_id = signal(1);

// Fetch data when user_id changes
let user = create_resource(
    move || user_id.get(),
    |id| async move {
        fetch_user(id).await.map_err(|e| e.to_string())
    }
);

// Check loading state
if user.loading() { /* show spinner */ }
if let Some(data) = user.data() { /* render user */ }
if let Some(err) = user.error() { /* show error */ }
```

## Zone-less Change Detection

Ferric uses a zone-less change detection system. No Zone.js monkey-patching - just explicit, signal-driven updates.

### ChangeDetectorRef

```rust
use ferric::change_detection::*;

// Each component gets a change detector
let cd = ChangeDetectorRef::on_push(); // OnPush strategy

// Mark for check (propagates to ancestors)
cd.mark_for_check();

// Immediate synchronous detection
cd.detect_changes();

// Detach from change detection (e.g., for hidden components)
cd.detach();
cd.reattach();
```

### Change Detection Strategies

```rust
// Default: Checked on every tick
let cd = ChangeDetectorRef::default_strategy();

// OnPush: Only checked when inputs change or explicitly marked
let cd = ChangeDetectorRef::on_push();
```

### Scheduler

```rust
// Schedule work for next microtask (batched)
schedule(|| {
    // This runs in next microtask
});

// Run when app is stable (no pending updates)
schedule_on_stable(|| {
    // Good for assertions, measurements
});
```

### ApplicationRef

```rust
// Root-level change detection
let app = ApplicationRef::new();

// Bootstrap a component
let root_cd = app.bootstrap(|cd| {
    // Create and configure component
});

// Manual tick
app.tick();

// Schedule tick (batched)
app.schedule_tick();

// Run outside change detection
app.run_outside_change_detection(|| {
    // Updates here won't trigger CD
});
```

## Routing

```rust
use ferric::router::*;

let routes = vec![
    Route::new("/").component("app-home"),
    Route::new("/users").component("app-users").children(vec![
        Route::new(":id").component("app-user-detail"),
    ]),
    Route::new("**").redirect_to("/"),
];

let router = Router::new(routes);
router.navigate("/users/123")?;
```

## License

MIT

