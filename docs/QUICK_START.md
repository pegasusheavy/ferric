# Ferric Quick Start Guide

Get up and running with Ferric in 5 minutes.

## Prerequisites

- Rust 1.75 or later
- wasm-pack
- Node.js 18+ (for development)

## Installation

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

### 2. Install wasm-pack

```bash
cargo install wasm-pack
```

### 3. Install Ferric CLI

```bash
cargo install ferric-cli
```

## Create Your First App

### 1. Generate a New Project

```bash
ferric new my-app
cd my-app
```

### 2. Project Structure

```
my-app/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── components/
├── public/
│   └── index.html
└── ferric.toml
```

### 3. Create a Component

Edit `src/lib.rs`:

```rust
use ferric::prelude::*;
use wasm_bindgen::prelude::*;

#[component(selector = "hello-world")]
struct HelloComponent {
    name: Signal<String>,
}

impl Injectable for HelloComponent {
    fn create(_injector: &Injector) -> Self {
        Self {
            name: signal("World".to_string()),
        }
    }
}

impl Component for HelloComponent {
    fn template(&self) -> String {
        format!(r#"
            <div class="hello">
                <h1>Hello, {}!</h1>
                <input type="text" value="{}" />
            </div>
        "#, self.name.get(), self.name.get())
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let injector = Injector::root();
    let component = injector.resolve::<HelloComponent>().unwrap();
    component.mount("#app");
}
```

### 4. Run Development Server

```bash
ferric serve
```

Open `http://localhost:8080` in your browser!

## Core Concepts

### Signals (Reactive State)

```rust
// Create a signal
let count = signal(0);

// Read value
let value = count.get();

// Update value
count.set(42);

// Update with function
count.update(|n| *n += 1);
```

### Computed Values

```rust
let count = signal(10);
let doubled = computed(|| count.get() * 2);

println!("{}", doubled.get()); // 20
count.set(20);
println!("{}", doubled.get()); // 40 (auto-updated!)
```

### Effects (Side Effects)

```rust
let count = signal(0);

effect(|| {
    println!("Count changed: {}", count.get());
});

count.set(1); // Prints: "Count changed: 1"
```

### Components

```rust
#[component(selector = "my-counter")]
struct CounterComponent {
    count: Signal<i32>,
}

impl Injectable for CounterComponent {
    fn create(_: &Injector) -> Self {
        Self { count: signal(0) }
    }
}

impl CounterComponent {
    fn increment(&self) {
        self.count.update(|n| *n += 1);
    }
}

impl Component for CounterComponent {
    fn template(&self) -> String {
        html! {
            <div>
                <h2>{self.count.get()}</h2>
                <button (click)="increment()">+</button>
            </div>
        }
    }
}
```

### Dependency Injection

```rust
#[injectable]
struct DataService {
    data: Signal<Vec<String>>,
}

impl Injectable for DataService {
    fn create(_: &Injector) -> Self {
        Self {
            data: signal(vec![]),
        }
    }
}

// Use in component
#[component(selector = "app-root")]
struct AppComponent {
    data_service: Rc<DataService>,
}

impl Injectable for AppComponent {
    fn create(injector: &Injector) -> Self {
        Self {
            data_service: injector.resolve_required::<DataService>(),
        }
    }
}
```

## Next Steps

1. **[Core Concepts](ARCHITECTURE.md)** - Learn about framework architecture
2. **[Components Guide](COMPONENTS_GUIDE.md)** - Deep dive into components
3. **[Routing](ROUTER_GUIDE.md)** - Add navigation to your app
4. **[Forms](FORMS_GUIDE.md)** - Build reactive forms
5. **[HTTP Client](HTTP_CLIENT_GUIDE.md)** - Make API requests
6. **[Examples](../examples/)** - Explore example applications

## Common Patterns

### Component Communication

```rust
// Parent passes data to child via inputs
#[component(selector = "child-component")]
struct ChildComponent {
    #[input]
    data: Signal<String>,
}

// Child emits events to parent via outputs
#[component(selector = "child-component")]
struct ChildComponent {
    #[output]
    on_change: EventEmitter<String>,
}

impl ChildComponent {
    fn notify(&self) {
        self.on_change.emit("Changed!".to_string());
    }
}
```

### Services

```rust
#[injectable]
struct UserService {
    http: Rc<InjectableHttpClient>,
    users: Signal<Vec<User>>,
}

impl UserService {
    async fn load_users(&self) {
        let response = self.http.get("/api/users").send().await.unwrap();
        let users: Vec<User> = response.json().unwrap();
        self.users.set(users);
    }
}
```

### Routing

```rust
let routes = vec![
    Route {
        path: "/".to_string(),
        component: Some("home-component".to_string()),
        ..Default::default()
    },
    Route {
        path: "/about".to_string(),
        component: Some("about-component".to_string()),
        ..Default::default()
    },
];

let router = Router::new(routes);
router.navigate("/about");
```

## Helpful Commands

```bash
# Create new project
ferric new my-app

# Run dev server
ferric serve

# Build for production
ferric build --release

# Run tests
cargo test --target wasm32-unknown-unknown

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Getting Help

- 📚 [Full Documentation](https://ferric-rs.github.io/docs)
- 💬 [GitHub Discussions](https://github.com/pegasusheavy/ferric/discussions)
- 🐛 [Issue Tracker](https://github.com/pegasusheavy/ferric/issues)
- 📖 [Examples](https://github.com/pegasusheavy/ferric/tree/main/examples)

## License

MIT

