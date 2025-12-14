# Documentation Testing Guide

Complete guide to writing and maintaining doctests in Ferric.

## What are Doctests?

Doctests are executable code examples embedded in documentation comments (`///`). They serve dual purposes:

1. **Documentation**: Show users how to use your code
2. **Testing**: Ensure examples stay up-to-date and correct

## Benefits

✅ **Accuracy**: Examples are tested automatically
✅ **Up-to-date**: Compiler ensures docs match code
✅ **Confidence**: Users trust working examples
✅ **Discoverability**: Examples appear in generated docs
✅ **Free Tests**: Documentation doubles as test suite

## Writing Doctests

### Basic Example

```rust
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// use my_crate::add;
///
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Module-level Examples

```rust
//! This module provides reactive primitives.
//!
//! # Examples
//!
//! ```
//! use ferric_core::reactive::signal;
//!
//! let count = signal(0);
//! count.set(42);
//! assert_eq!(count.get(), 42);
//! ```
```

### Multiple Examples

```rust
/// Creates a signal with an initial value.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use ferric_core::reactive::signal;
///
/// let count = signal(0);
/// assert_eq!(count.get(), 0);
/// ```
///
/// With custom types:
///
/// ```
/// use ferric_core::reactive::signal;
///
/// let name = signal("Alice".to_string());
/// assert_eq!(name.get(), "Alice");
/// ```
pub fn signal<T>(value: T) -> Signal<T> {
    Signal::new(value)
}
```

## Doctest Attributes

### Should Panic

```rust
/// Divides two numbers.
///
/// # Panics
///
/// Panics if divisor is zero:
///
/// ```should_panic
/// use my_crate::divide;
///
/// divide(10, 0); // This will panic!
/// ```
pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division by zero!");
    }
    a / b
}
```

### Compile Fail

```rust
/// This function requires Clone.
///
/// ```compile_fail
/// use my_crate::print_twice;
///
/// struct NotClone;
/// print_twice(NotClone); // Won't compile!
/// ```
pub fn print_twice<T: Clone>(value: T) {
    println!("{:?}", value);
    println!("{:?}", value);
}
```

### No Run

For expensive or platform-specific code:

```rust
/// Starts the HTTP server.
///
/// # Examples
///
/// ```no_run
/// use my_server::start;
///
/// start("0.0.0.0:8080"); // Don't actually start server in tests
/// ```
pub fn start(addr: &str) {
    // Server implementation
}
```

### Ignore

Skip tests temporarily:

```rust
/// Not yet implemented.
///
/// # Examples
///
/// ```ignore
/// use my_crate::future_feature;
///
/// future_feature(); // TODO: implement this
/// ```
pub fn future_feature() {
    todo!()
}
```

## Advanced Patterns

### Hidden Lines

Lines starting with `#` are hidden in documentation but run in tests:

```rust
/// Processes a request.
///
/// # Examples
///
/// ```
/// # use ferric_http::Request;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let request = Request::new("GET", "/api/users");
/// let response = request.send()?;
/// assert_eq!(response.status(), 200);
/// # Ok(())
/// # }
/// ```
```

**Displayed as:**

```rust
let request = Request::new("GET", "/api/users");
let response = request.send()?;
assert_eq!(response.status(), 200);
```

### Setup and Teardown

```rust
/// # Examples
///
/// ```
/// # use ferric_core::di::{Injector, Injectable, ProviderBuilder};
/// # #[derive(Clone)]
/// # struct MyService { value: i32 }
/// # impl Injectable for MyService {
/// #     fn create(_: &Injector) -> Self { Self { value: 42 } }
/// # }
/// let injector = Injector::root();
/// injector.provide(ProviderBuilder::<MyService>::new().build());
///
/// let service = injector.resolve_required::<MyService>();
/// assert_eq!(service.value, 42);
/// ```
```

### Shared Code

For complex setup, use modules:

```rust
/// # Examples
///
/// ```
/// # mod test_utils {
/// #     use ferric_core::di::{Injector, Injectable};
/// #     pub fn setup() -> Injector {
/// #         let injector = Injector::root();
/// #         // Complex setup...
/// #         injector
/// #     }
/// # }
/// #
/// use test_utils::setup;
///
/// let injector = setup();
/// // Use injector...
/// ```
```

## Testing Async Code

### WASM Target

```rust
/// # Examples
///
/// ```
/// # use ferric_core::async_support::spawn_local;
/// # #[cfg(target_arch = "wasm32")]
/// # use wasm_bindgen_test::*;
/// #
/// # #[cfg(target_arch = "wasm32")]
/// # #[wasm_bindgen_test]
/// async fn test_async_function() {
///     let result = my_async_fn().await;
///     assert_eq!(result, expected);
/// }
/// ```
```

### Tokio Runtime

```rust
/// # Examples
///
/// ```
/// # #[cfg(not(target_arch = "wasm32"))]
/// # #[tokio::test]
/// async fn test_async_http() {
///     let response = fetch_data().await.unwrap();
///     assert!(response.is_success());
/// }
/// ```
```

## Best Practices

### 1. Test Real Use Cases

```rust
// ✅ Good: Shows realistic usage
/// # Examples
///
/// ```
/// use ferric_core::reactive::{signal, computed, effect};
///
/// let count = signal(0);
/// let doubled = computed(move || count.get() * 2);
///
/// effect(move || {
///     println!("Doubled: {}", doubled.get());
/// });
///
/// count.set(5); // Prints: "Doubled: 10"
/// ```

// ❌ Bad: Trivial example
/// # Examples
///
/// ```
/// let x = 5;
/// ```
```

### 2. Keep Examples Focused

```rust
// ✅ Good: One concept per example
/// Sets the signal value.
///
/// # Examples
///
/// ```
/// use ferric_core::reactive::signal;
///
/// let count = signal(0);
/// count.set(42);
/// assert_eq!(count.get(), 42);
/// ```

// ❌ Bad: Too much in one example
/// # Examples
///
/// ```
/// // 50 lines of setup...
/// // Multiple unrelated concepts...
/// ```
```

### 3. Show Error Handling

```rust
/// # Examples
///
/// ```
/// use ferric_http::Client;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new();
/// let response = client.get("https://api.example.com/data").send().await?;
///
/// if response.is_success() {
///     let data = response.json()?;
///     // Process data...
/// }
/// # Ok(())
/// # }
/// ```
```

### 4. Use Type Annotations When Helpful

```rust
/// # Examples
///
/// ```
/// use ferric_core::reactive::signal;
///
/// // Type annotation helps readers understand
/// let count: Signal<i32> = signal(0);
/// ```
```

### 5. Document Panics

```rust
/// # Panics
///
/// Panics if the index is out of bounds:
///
/// ```should_panic
/// let vec = vec![1, 2, 3];
/// let item = vec[10]; // Panics!
/// ```
```

## Running Doctests

### All Doctests

```bash
cargo test --doc
```

### Specific Crate

```bash
cargo test --doc -p ferric-core
```

### Specific Module

```bash
cargo test --doc -p ferric-core -- reactive
```

### With Output

```bash
cargo test --doc -- --nocapture
```

## CI Integration

### GitHub Actions

```yaml
- name: Run doctests
  run: |
    cargo test --doc --all
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

cargo test --doc --quiet || {
    echo "Doctests failed!"
    exit 1
}
```

## Troubleshooting

### Doctest Won't Compile

1. Check imports are correct
2. Verify types match
3. Use hidden setup code (`#`)
4. Check for missing dependencies in `Cargo.toml`

### Example Needs External Setup

Use `no_run`:

```rust
/// ```no_run
/// // Code that needs database, network, etc.
/// ```
```

### Platform-Specific Code

```rust
/// # Examples
///
/// ```
/// # #[cfg(target_arch = "wasm32")]
/// # {
/// // WASM-specific code
/// # }
/// ```
```

## Coverage Goals

- **Public APIs**: 100% doctest coverage
- **Common patterns**: Multiple examples
- **Error cases**: Document failure modes
- **Complex features**: Step-by-step examples

## Ferric-Specific Patterns

### Signals

```rust
/// ```
/// use ferric_core::reactive::signal;
///
/// let count = signal(0);
/// count.update(|n| *n += 1);
/// assert_eq!(count.get(), 1);
/// ```
```

### Dependency Injection

```rust
/// ```
/// use ferric_core::di::{Injector, Injectable, ProviderBuilder};
///
/// #[derive(Clone)]
/// struct MyService;
///
/// impl Injectable for MyService {
///     fn create(_: &Injector) -> Self { MyService }
/// }
///
/// let injector = Injector::root();
/// injector.provide(ProviderBuilder::<MyService>::new().build());
/// ```
```

### Components

```rust
/// ```ignore
/// use ferric_core::prelude::*;
///
/// #[component(selector = "my-comp")]
/// struct MyComponent {
///     count: Signal<i32>,
/// }
/// ```
```

## Tools

### cargo-doctest

```bash
cargo install cargo-doctest
cargo doctest
```

### Rustdoc

```bash
cargo doc --open
```

## Resources

- [Rust Book - Documentation Tests](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments-as-tests)
- [Rustdoc Guide](https://doc.rust-lang.org/rustdoc/)
- [API Guidelines - Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)

## License

MIT

