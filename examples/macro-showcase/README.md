# Ferric Macros Showcase

This example demonstrates all the new utility macros added to the Ferric framework.

## What's Included

### Reactive Macros
- `signal!` - Create reactive signals
- `computed!` - Create computed values
- `effect!` - Create side effects
- `batch!` - Batch signal updates
- `memo!` - Memoize expensive computations
- `watch!` - Watch for value changes
- `lazy!` - Lazy static initialization

### Template Macros
- `html!` - Parse HTML templates
- `css!` - Define CSS styles
- `selector!` - Validate component selectors

### Event Macros
- `on!` - Create event handlers
- `emit!` - Emit events

### DI Macros
- `inject!` - Inject dependencies
- `provide!` - Register providers

### Testing Macros
- `component_test!` - Component test setup
- `async_test!` - Async test setup

### Forms Macros
- `#[derive(TypedForm)]` - Type-safe forms
- `build_form!` - Declarative form building
- `reactive_form!` - Reactive forms with signals
- `compose_validators!` - Compose validators
- `required_if!` - Conditional validation
- `match_field!` - Cross-field validation
- `async_validator!` - Async validation

## Building

```bash
# Check compilation
cargo check -p macro-showcase

# Build for WASM
cargo build -p macro-showcase --target wasm32-unknown-unknown
```

## Running

This is a library crate demonstrating macro usage. To see the macros in action:

1. Look at the source code in `src/lib.rs`
2. Run the tests: `cargo test -p macro-showcase`
3. Build as WASM and load in a browser

## Key Examples

### Reactive State Management

```rust
let count = signal!(0);
let doubled = computed!(|| count.get() * 2);

effect!(|| {
    println!("Count: {}", count.get());
});

batch!(|| {
    count.set(10);
    // Multiple updates batched together
});
```

### Type-Safe Forms

```rust
#[derive(TypedForm)]
struct User {
    username: String,
    email: String,
    age: u32,
}

let form = UserForm::from_value(user);
form.username.set_value("newname".to_string());
let updated_user: User = form.into();
```

### Declarative Form Building

```rust
let form = build_form! {
    email: FormControl::text("").with_validators(compose_validators![
        required(),
        email(),
    ]),
    password: FormControl::password("").with_validators(compose_validators![
        required(),
        min_length(8),
    ]),
};
```

### Conditional Validation

```rust
let is_company = signal!(false);

let form = build_form! {
    company_name: FormControl::text("").with_validator(
        required_if!(is_company.get())
    ),
};
```

## Documentation

For complete documentation, see:

- [Ferric Macros Guide](../../ferric-macros/MACROS_GUIDE.md)
- [Forms Macros Guide](../../ferric-forms-macros/FORMS_MACROS_GUIDE.md)
- [Macros Summary](../../MACROS_SUMMARY.md)

