# Ferric Reactivity Demo

A comprehensive demonstration of Ferric's reactive system featuring zone-less change detection.

**Styled with Tailwind CSS** - Uses the Tailwind CDN for a beautiful dark theme UI with no custom CSS.

## Features Demonstrated

### 1. Counter Demo (`counter.rs`)

Basic reactive primitives:

```rust
// Signals: Mutable reactive state
let count = signal(0);
count.set(10);
count.update(|n| n + 1);

// Computed: Derived values with auto-tracking
let doubled = computed(move || count.get() * 2);

// Effect: Side effects that run when dependencies change
let _fx = effect(move || {
    console::log_1(&format!("Count: {}", count.get()).into());
});

// Batch: Coalesce multiple updates into one notification
batch(|| {
    for _ in 0..10 {
        count.update(|n| n + 1);
    }
}); // Only ONE effect run!
```

### 2. Shopping Cart Demo (`shopping_cart.rs`)

Complex state management:

```rust
// Multiple computed values forming a dependency chain
let subtotal = computed(move || {
    cart.get().iter()
        .map(|i| i.product.price * i.quantity as f64)
        .sum()
});

let discount = computed(move || {
    subtotal.get() * discount_rate.get()
});

let total = computed(move || {
    subtotal.get() - discount.get()
});

// Watch: Observe changes with old/new comparison
let _watch = watch(
    move || cart.get().len(),
    |new_len, old_len| {
        console::log_1(&format!(
            "Cart: {} → {} items",
            old_len.unwrap_or(0),
            new_len
        ).into());
    }
);

// Derived: Transform a signal's output
let total_display = derived(&total, |t| format!("${:.2}", t));
```

### 3. Async Search Demo (`async_search.rs`)

Async data fetching patterns:

```rust
// Debounced input
let query = signal(String::new());
let debounced_query = signal(String::new());

// Debounce effect with cleanup
let _debounce = effect(move || {
    let q = query.get();

    // Clear existing timeout
    if let Some(id) = timeout_id.borrow_mut().take() {
        window.clear_timeout_with_handle(id);
    }

    // Schedule update after 300ms
    let id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        300,
    );
    *timeout_id.borrow_mut() = Some(id);
});

// Search effect - runs async operation
let _search = effect(move || {
    let q = debounced_query.get();
    is_loading.set(true);

    spawn_local(async move {
        match search(q).await {
            Ok(results) => results_signal.set(results),
            Err(e) => error_signal.set(Some(e)),
        }
        is_loading.set(false);
    });
});
```

### 4. Timer Demo (`timer.rs`)

Effect cleanup and `untracked()`:

```rust
// Effect with interval cleanup
let _timer = effect(move || {
    let running = is_running.get();

    // Clear existing interval (cleanup)
    if let Some(id) = interval_id.borrow_mut().take() {
        window.clear_interval_with_handle(id);
    }

    if running {
        let count = counter.clone();

        // Use untracked to read without creating dependency
        let closure = Closure::wrap(Box::new(move || {
            let current = untracked(|| count.get());
            count.set(current + 1);
        }) as Box<dyn Fn()>);

        let id = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            1000,
        );
        *interval_id.borrow_mut() = Some(id);
    }
});
```

### 5. Form Validation Demo (`form_validation.rs`)

Reactive form state:

```rust
// Field state with touched/dirty tracking
#[derive(Clone)]
struct FieldState {
    value: String,
    touched: bool,
    dirty: bool,
}

let username = signal(FieldState::default());

// Computed validation (only shows errors when touched)
let username_errors = computed(move || {
    let state = username.get();
    if !state.touched {
        return vec![];
    }
    validate_username(&state)
});

// Overall form validity
let is_valid = computed(move || {
    validate_username(&username.get()).is_empty() &&
    validate_email(&email.get()).is_empty() &&
    validate_password(&password.get()).is_empty()
});
```

## Running the Demo

```bash
# From the project root
cd examples/reactivity-demo

# Build with wasm-pack
wasm-pack build --target web

# Serve (using any static server)
python3 -m http.server 8080
# or
npx serve .
```

Open `http://localhost:8080` in your browser.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Reactive Data Flow                            │
│                                                                  │
│   User Action ──► Signal.set() ──► Computed recalculates        │
│                        │                    │                    │
│                        ▼                    ▼                    │
│                   Effects run          DOM Updates               │
│                        │                                         │
│                        ▼                                         │
│           Side effects (console, storage, etc.)                  │
└─────────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Zone-less Reactivity

Unlike Angular's Zone.js, Ferric uses explicit signal-based reactivity:

- No monkey-patching of async APIs
- Clear dependency tracking
- Predictable update timing
- Manual control when needed

### Automatic Dependency Tracking

```rust
// Computed automatically tracks which signals it reads
let derived = computed(move || {
    // Reading a.get() and b.get() creates dependencies
    a.get() + b.get()
});
```

### Batching

```rust
// Multiple updates trigger only ONE notification
batch(|| {
    x.set(1);
    y.set(2);
    z.set(3);
}); // Effects/computed recalculate once
```

### Untracked Reads

```rust
// Read a signal without creating a dependency
let value = untracked(|| signal.get());
```

## License

MIT

