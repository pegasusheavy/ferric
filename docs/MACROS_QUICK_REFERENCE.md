# Ferric Macros Quick Reference

One-page reference for all Ferric utility macros.

## 🔄 Reactive Macros

```rust
// Create signals
let count = signal!(0);

// Computed values
let doubled = computed!(|| count.get() * 2);

// Side effects
effect!(|| println!("{}", count.get()));

// Batch updates
batch!(|| {
    count.set(10);
    name.set("Bob".to_string());
});

// Memoize expensive computations
let result = memo!(|| fibonacci(50));

// Watch for changes
watch!(|| if count.get() > 10 { alert(); });

// Lazy static
let config = lazy!(|| Config::load());
```

## 📝 Template Macros

```rust
// Parse HTML
let template = html!(r#"<div>{{ title }}</div>"#);

// Define CSS
let styles = css!(r#".app { padding: 20px; }"#);

// Validate selector
let selector = selector!("app-component");
```

## 🎯 Event Macros

```rust
// Event handler
let handler = on!(click => |e| println!("Clicked!"));

// Emit event
emit!(self.count_changed, count);
```

## 💉 DI Macros

```rust
// Inject dependency
let service = inject!(UserService);

// Register provider
provide!(UserService);
```

## 🧪 Testing Macros

```rust
// Component test
component_test! {
    fn test() {
        let fixture = test_bed.create_component::<MyComponent>();
        assert_eq!(fixture.instance().value, 42);
    }
}

// Async test
async_test! {
    fn test() async {
        let result = fetch_data().await;
        assert!(result.is_ok());
    }
}
```

## 📋 Forms Macros

```rust
// Type-safe form
#[derive(TypedForm)]
struct User {
    name: String,
    email: String,
    age: u32,
}
let form = UserForm::from_value(user);

// Build form declaratively
let form = build_form! {
    email: FormControl::text("").with_validators(compose_validators![
        required(),
        email(),
    ]),
};

// Reactive form
let form = reactive_form! {
    name: FormControl::text(""),
    age: FormControl::number(0),
};

// Compose validators
let validators = compose_validators![
    required(),
    min_length(3),
    max_length(50),
];

// Conditional validation
let validator = required_if!(is_company.get());

// Cross-field validation
let validator = match_field!("password");

// Async validation
let validator = async_validator!(|v| async {
    check_availability(v).await
});
```

## 📦 Complete Component Example

```rust
#[component(
    selector = "user-profile",
    template = html!(r#"
        <div class="profile">
            <h2>{{ name }}</h2>
            <button (click)="save()">Save</button>
        </div>
    "#),
    styles = css!(r#"
        .profile { padding: 20px; }
    "#)
)]
pub struct UserProfile {
    form: FormGroup,
    user: Signal<User>,
}

impl UserProfile {
    pub fn new() -> Self {
        let user = signal!(User::default());
        
        let form = build_form! {
            name: FormControl::text("").with_validators(compose_validators![
                required(),
                min_length(2),
            ]),
            email: FormControl::text("").with_validators(compose_validators![
                required(),
                email(),
            ]),
        };
        
        effect!(|| {
            web_sys::console::log_1(&format!("User: {:?}", user.get()).into());
        });
        
        Self { form, user }
    }
    
    pub fn save(&self) {
        if self.form.is_valid() {
            batch!(|| {
                self.user.update(|u| u.saved = true);
                emit!(self.save_complete, self.user.get());
            });
        }
    }
}
```

## 🔗 Links

- [Full Macros Guide](ferric-macros/MACROS_GUIDE.md)
- [Forms Macros Guide](ferric-forms-macros/FORMS_MACROS_GUIDE.md)
- [Macros Summary](MACROS_SUMMARY.md)
- [Example Showcase](examples/macro-showcase/)

## 💡 Tips

1. Use `signal!` instead of `signal()` for cleaner code
2. Prefer `computed!` for derived state
3. Use `batch!` when updating multiple signals
4. Use `#[derive(TypedForm)]` for type-safe forms
5. Compose validators with `compose_validators!`
6. Use `component_test!` for easier test setup

