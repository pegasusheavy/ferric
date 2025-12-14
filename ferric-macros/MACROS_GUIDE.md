# Ferric Macros Guide

Comprehensive guide to all available macros in the Ferric framework.

## Table of Contents

1. [Component Macros](#component-macros)
2. [Reactive Macros](#reactive-macros)
3. [Template Macros](#template-macros)
4. [Event Macros](#event-macros)
5. [DI Macros](#di-macros)
6. [Testing Macros](#testing-macros)

## Component Macros

### `#[component]`

Define a component with metadata:

```rust
#[component(
    selector = "app-counter",
    template = r#"
        <div class="counter">
            <span>{{ count }}</span>
            <button (click)="increment()">+</button>
        </div>
    "#,
    styles = ".counter { padding: 20px; }"
)]
pub struct CounterComponent {
    count: Signal<i32>,
}
```

### `#[injectable]`

Mark a service as injectable:

```rust
#[injectable]
pub struct UserService {
    http: Arc<HttpClient>,
}
```

### `#[input]` / `#[output]`

Define component inputs and outputs:

```rust
#[component(selector = "user-card")]
pub struct UserCard {
    #[input]
    user: User,

    #[output]
    user_selected: EventEmitter<User>,
}
```

## Reactive Macros

### `signal!`

Create a reactive signal:

```rust
let count = signal!(0);
let name = signal!("Alice".to_string());
```

### `computed!`

Create a computed value:

```rust
let doubled = computed!(|| count.get() * 2);
let full_name = computed!(|| format!("{} {}", first.get(), last.get()));
```

### `effect!`

Create a side effect:

```rust
effect!(|| {
    println!("Count changed to: {}", count.get());
});
```

### `batch!`

Batch multiple updates:

```rust
batch!(|| {
    count.set(10);
    name.set("Bob".to_string());
    active.set(true);
});
```

### `memo!`

Create a memoized value (cached):

```rust
let expensive_result = memo!(|| {
    compute_fibonacci(50)
});
```

### `watch!`

Watch a value for changes:

```rust
watch!(|| {
    if count.get() > 100 {
        alert("Count exceeded 100!");
    }
});
```

### `lazy!`

Create a lazy-initialized static:

```rust
let config = lazy!(|| Config::load());
```

## Template Macros

### `html!`

Parse HTML template:

```rust
let template = html!(r#"
    <div class="app">
        <h1>{{ title }}</h1>
        <p>{{ description }}</p>
    </div>
"#);
```

### `css!`

Define CSS styles:

```rust
let styles = css!(r#"
    .app { padding: 20px; }
    h1 { color: blue; }
"#);
```

### `selector!`

Validate component selector:

```rust
let selector = selector!("app-my-component");
```

## Event Macros

### `on!`

Create event handler:

```rust
let handler = on!(click => |event| {
    println!("Clicked!");
});

let key_handler = on!(keydown => |event| {
    if event.key() == "Enter" {
        submit_form();
    }
});
```

### `emit!`

Emit an event:

```rust
emit!(self.count_changed, count);
emit!(self.user_selected, user.clone());
```

## DI Macros

### `inject!`

Inject a dependency:

```rust
let user_service = inject!(UserService);
let http_client = inject!(HttpClient);
```

### `provide!`

Register a provider:

```rust
provide!(UserService);
provide!(HttpClient);
```

## Testing Macros

### `component_test!`

Create a component test:

```rust
component_test! {
    fn test_counter_increment() {
        let fixture = test_bed.create_component::<CounterComponent>();
        let component = fixture.instance();

        component.increment();

        assert_eq!(component.count.get(), 1);
    }
}
```

### `async_test!`

Create an async test:

```rust
async_test! {
    fn test_fetch_users() {
        let service = UserService::new();
        let users = service.fetch_all().await.unwrap();

        assert!(!users.is_empty());
    }
}
```

## Advanced Examples

### Complete Component with Macros

```rust
use ferric_core::prelude::*;

#[component(
    selector = "user-profile",
    template = html!(r#"
        <div class="profile">
            <h2>{{ fullName }}</h2>
            <p>Email: {{ email }}</p>
            <button (click)="toggleEdit()">Edit</button>
        </div>
    "#),
    styles = css!(r#"
        .profile { padding: 20px; border: 1px solid #ccc; }
        h2 { margin: 0 0 10px 0; }
    "#)
)]
pub struct UserProfile {
    #[input]
    user: User,

    #[output]
    edit_requested: EventEmitter<User>,

    is_editing: Signal<bool>,
    full_name: Computed<String>,
}

impl UserProfile {
    pub fn new() -> Self {
        let is_editing = signal!(false);
        let user_signal = signal!(User::default());

        let full_name = computed!(|| {
            let user = user_signal.get();
            format!("{} {}", user.first_name, user.last_name)
        });

        effect!(|| {
            if is_editing.get() {
                println!("Editing mode enabled");
            }
        });

        Self {
            user: User::default(),
            edit_requested: EventEmitter::new(),
            is_editing,
            full_name,
        }
    }

    pub fn toggle_edit(&self) {
        self.is_editing.update(|editing| !editing);
        emit!(self.edit_requested, self.user.clone());
    }
}
```

### Reactive Form with Macros

```rust
use ferric_forms::prelude::*;

#[injectable]
pub struct LoginService {
    http: Arc<HttpClient>,
    form: FormGroup,
}

impl LoginService {
    pub fn new() -> Self {
        let form = build_form! {
            email: FormControl::text("")
                .with_validators(compose_validators![
                    required(),
                    email(),
                ]),
            password: FormControl::password("")
                .with_validators(compose_validators![
                    required(),
                    min_length(8),
                ]),
        };

        effect!(|| {
            if form.is_valid() {
                println!("Form is valid!");
            }
        });

        Self {
            http: inject!(HttpClient),
            form,
        }
    }

    pub async fn login(&self) -> Result<User, Error> {
        if !self.form.is_valid() {
            return Err(Error::ValidationFailed);
        }

        let email = self.form.get_value("email");
        let password = self.form.get_value("password");

        self.http.post("/api/login")
            .json(&LoginRequest { email, password })
            .send()
            .await?
            .json()
    }
}
```

## Best Practices

1. **Use `signal!` for reactive state**: Prefer signals over plain values for UI state
2. **Use `computed!` for derived values**: Avoid manual recalculation
3. **Batch related updates**: Use `batch!` when updating multiple signals
4. **Memoize expensive computations**: Use `memo!` for costly operations
5. **Use `inject!` in constructors**: Keep DI consistent
6. **Test with `component_test!`**: Leverage automatic TestBed setup

## Performance Tips

- Use `OnPush` change detection with signals
- Memoize expensive computed values
- Batch signal updates to minimize re-renders
- Use `lazy!` for one-time initializations
- Prefer `computed!` over `effect!` for derived state

## Migration Guide

### From Manual Signal Creation

```rust
// Before
let count = signal(0);
let doubled = computed(move || count.get() * 2);

// After
let count = signal!(0);
let doubled = computed!(|| count.get() * 2);
```

### From Manual Form Building

```rust
// Before
let mut group = FormGroup::new();
group.add_control("email", Rc::new(FormControl::text("")));
group.add_control("password", Rc::new(FormControl::password("")));

// After
let form = build_form! {
    email: FormControl::text(""),
    password: FormControl::password(""),
};
```

## See Also

- [Ferric Forms Macros Guide](../../ferric-forms-macros/FORMS_MACROS_GUIDE.md)
- [Component Guide](../docs/COMPONENTS.md)
- [Reactive Programming Guide](../docs/REACTIVITY.md)

