# Components Guide

Complete guide to creating and using components in Ferric.

## What is a Component?

A component is a reusable building block of your UI. It combines:
- **Template**: HTML structure
- **Styles**: CSS styling
- **Logic**: Rust code for behavior
- **State**: Reactive signals and computed values

## Creating a Component

### Basic Component

```rust
use ferric::prelude::*;

#[component(selector = "my-button")]
struct ButtonComponent {
    label: Signal<String>,
    count: Signal<i32>,
}

impl Injectable for ButtonComponent {
    fn create(_injector: &Injector) -> Self {
        Self {
            label: signal("Click me".to_string()),
            count: signal(0),
        }
    }
}

impl ButtonComponent {
    fn handle_click(&self) {
        self.count.update(|n| *n += 1);
    }
}

impl Component for ButtonComponent {
    fn template(&self) -> String {
        html! {
            <button (click)="handle_click()">
                {self.label.get()} ({self.count.get()})
            </button>
        }
    }

    fn styles(&self) -> Vec<String> {
        vec![css! {
            button {
                padding: 0.75rem 1.5rem;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                border: none;
                border-radius: 8px;
                cursor: pointer;
                font-weight: 600;
            }

            button:hover {
                transform: translateY(-2px);
                box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
            }
        }]
    }
}
```

## Component Metadata

### Selector

The CSS selector used to identify your component:

```rust
#[component(selector = "app-header")]
//                      ^^^^^^^^^^^ use in HTML as <app-header>
```

### Template

Define your component's HTML structure:

```rust
impl Component for MyComponent {
    fn template(&self) -> String {
        html! {
            <div class="container">
                <h1>{self.title.get()}</h1>
                <p>{self.description.get()}</p>
            </div>
        }
    }
}
```

### Styles

Component-scoped CSS:

```rust
impl Component for MyComponent {
    fn styles(&self) -> Vec<String> {
        vec![css! {
            .container {
                padding: 2rem;
                background: white;
                border-radius: 10px;
            }
        }]
    }
}
```

### View Encapsulation

Control how styles are scoped:

```rust
#[component(
    selector = "my-component",
    encapsulation = ViewEncapsulation::Emulated
)]
```

Options:
- `Emulated` - Scope styles using attribute selectors (default)
- `ShadowDom` - Use Shadow DOM (browser support required)
- `None` - Global styles (no scoping)

## Inputs and Outputs

### Input Properties

Receive data from parent components:

```rust
#[component(selector = "user-card")]
struct UserCardComponent {
    #[input]
    username: Signal<String>,

    #[input(alias = "userEmail")]
    email: Signal<String>,
}

// Usage in parent template:
// <user-card [username]="currentUser" [userEmail]="email"></user-card>
```

### Output Events

Send events to parent components:

```rust
#[component(selector = "custom-button")]
struct CustomButtonComponent {
    #[output]
    on_click: EventEmitter<String>,
}

impl CustomButtonComponent {
    fn handle_click(&self) {
        self.on_click.emit("Button clicked!".to_string());
    }
}

// Usage in parent:
// <custom-button (onClick)="handleButtonClick($event)"></custom-button>
```

## Lifecycle Hooks

### OnInit

Called after component initialization:

```rust
impl OnInit for MyComponent {
    fn fe_on_init(&self) {
        // Load initial data
        self.load_data();
    }
}
```

### AfterViewInit

Called after the component's view is initialized:

```rust
impl AfterViewInit for MyComponent {
    fn fe_after_view_init(&self) {
        // DOM is ready, safe to query elements
        self.setup_third_party_lib();
    }
}
```

### OnDestroy

Called before component is destroyed:

```rust
impl OnDestroy for MyComponent {
    fn fe_on_destroy(&self) {
        // Cleanup subscriptions, timers, etc.
        self.cleanup();
    }
}
```

### Async Lifecycle

```rust
impl AsyncOnInit for MyComponent {
    async fn fe_async_on_init(&self) {
        let data = self.http.get("/api/data").send().await.unwrap();
        self.data.set(data.json().unwrap());
    }
}
```

## Component Communication

### Parent to Child (Inputs)

```rust
// Parent
#[component(selector = "parent")]
struct ParentComponent {
    user_data: Signal<String>,
}

impl Component for ParentComponent {
    fn template(&self) -> String {
        html! {
            <child-component [data]="user_data.get()"></child-component>
        }
    }
}

// Child
#[component(selector = "child-component")]
struct ChildComponent {
    #[input]
    data: Signal<String>,
}
```

### Child to Parent (Outputs)

```rust
// Child
#[component(selector = "child")]
struct ChildComponent {
    #[output]
    on_save: EventEmitter<String>,
}

impl ChildComponent {
    fn save(&self, value: String) {
        self.on_save.emit(value);
    }
}

// Parent
#[component(selector = "parent")]
struct ParentComponent;

impl ParentComponent {
    fn handle_save(&self, value: String) {
        println!("Saved: {}", value);
    }
}

impl Component for ParentComponent {
    fn template(&self) -> String {
        html! {
            <child-component (onSave)="handle_save($event)"></child-component>
        }
    }
}
```

### Service-based Communication

```rust
#[injectable]
struct DataService {
    shared_data: Signal<Vec<String>>,
}

// Both components can inject and use DataService
#[component(selector = "component-a")]
struct ComponentA {
    data_service: Rc<DataService>,
}

#[component(selector = "component-b")]
struct ComponentB {
    data_service: Rc<DataService>,
}
```

## Template Syntax

### Interpolation

```rust
html! {
    <h1>{self.title.get()}</h1>
    <p>Count: {self.count.get()}</p>
}
```

### Property Binding

```rust
html! {
    <input [value]="name.get()" />
    <img [src]="image_url.get()" />
    <button [disabled]="is_loading.get()">Submit</button>
}
```

### Event Binding

```rust
html! {
    <button (click)="handle_click()">Click</button>
    <input (input)="handle_input($event)" />
    <form (submit)="handle_submit($event)">...</form>
}
```

### Two-way Binding

```rust
html! {
    <input [(ngModel)]="name" />
}
```

### Structural Directives

```rust
// *if
html! {
    <div *if="show_content.get()">
        Content here
    </div>
}

// *for
html! {
    <ul>
        <li *for="item in items.get()">
            {item.name}
        </li>
    </ul>
}

// *switch
html! {
    <div *switch="view_mode.get()">
        <div *case="'list'">List View</div>
        <div *case="'grid'">Grid View</div>
        <div *default>Default View</div>
    </div>
}
```

## Advanced Topics

### ViewChild / ViewChildren

Query child components or elements:

```rust
#[component(selector = "parent")]
struct ParentComponent {
    #[view_child(selector = "child-component")]
    child: Signal<Option<Rc<ChildComponent>>>,
}

impl AfterViewInit for ParentComponent {
    fn fe_after_view_init(&self) {
        if let Some(child) = self.child.get().as_ref() {
            child.do_something();
        }
    }
}
```

### Content Projection

```rust
impl Component for CardComponent {
    fn template(&self) -> String {
        html! {
            <div class="card">
                <div class="header">
                    <ng-content select="card-header"></ng-content>
                </div>
                <div class="body">
                    <ng-content></ng-content>
                </div>
            </div>
        }
    }
}

// Usage:
// <card-component>
//     <card-header>Title</card-header>
//     <p>Body content</p>
// </card-component>
```

### Host Bindings

Bind to the host element:

```rust
#[component(selector = "highlight")]
struct HighlightComponent {
    #[host_binding(attr = "class")]
    css_class: Signal<String>,

    #[host_binding(style = "background-color")]
    background: Signal<String>,
}

impl HighlightComponent {
    fn set_highlight(&self, color: String) {
        self.background.set(color);
    }
}
```

### Change Detection

Control when your component updates:

```rust
#[component(
    selector = "my-component",
    change_detection = ChangeDetectionStrategy::OnPush
)]
```

Strategies:
- `Default` - Check on every change detection cycle
- `OnPush` - Only check when inputs change or events fire

## Best Practices

### 1. Keep Components Small

```rust
// ✅ Good: Single responsibility
#[component(selector = "user-avatar")]
struct UserAvatarComponent { /* ... */ }

// ❌ Bad: Too many responsibilities
#[component(selector = "user-profile-dashboard-with-settings")]
struct MassiveComponent { /* ... */ }
```

### 2. Use Services for Logic

```rust
// ✅ Good: Logic in service
#[injectable]
struct UserService {
    fn calculate_score(&self, user: &User) -> i32 { /* ... */ }
}

// ❌ Bad: Too much logic in component
#[component(selector = "user-card")]
struct UserCardComponent {
    fn complex_calculation(&self) { /* ... */ }
}
```

### 3. Avoid Subscriptions in Components

```rust
// ✅ Good: Use computed or effects
let doubled = computed(|| count.get() * 2);

// ❌ Bad: Manual subscriptions
let subscription = count.subscribe(|val| { /* ... */ });
```

### 4. Clean Up Resources

```rust
impl OnDestroy for MyComponent {
    fn fe_on_destroy(&self) {
        // Cancel pending requests
        self.cancel_token.cancel();

        // Clear intervals
        self.timer_handle.clear();
    }
}
```

## Common Patterns

### Smart/Dumb Components

```rust
// Smart (Container) - Has logic and services
#[component(selector = "user-list-container")]
struct UserListContainer {
    user_service: Rc<UserService>,
    users: Signal<Vec<User>>,
}

// Dumb (Presentational) - Only displays data
#[component(selector = "user-list")]
struct UserListComponent {
    #[input]
    users: Signal<Vec<User>>,

    #[output]
    on_select: EventEmitter<User>,
}
```

### Loading States

```rust
#[component(selector = "data-loader")]
struct DataLoaderComponent {
    is_loading: Signal<bool>,
    data: Signal<Option<Data>>,
    error: Signal<Option<String>>,
}

impl Component for DataLoaderComponent {
    fn template(&self) -> String {
        html! {
            <div *if="is_loading.get()">Loading...</div>
            <div *if="error.get().is_some()">Error: {error.get().unwrap()}</div>
            <div *if="data.get().is_some()">
                <!-- Display data -->
            </div>
        }
    }
}
```

### Form Components

```rust
#[component(selector = "user-form")]
struct UserFormComponent {
    form_builder: Rc<FormBuilder>,
    user_form: Signal<Option<Rc<FormGroup>>>,
}

impl Injectable for UserFormComponent {
    fn create(injector: &Injector) -> Self {
        let form_builder = injector.resolve_required::<FormBuilder>();

        let mut controls = HashMap::new();
        controls.insert("name".to_string(), Rc::new(form_builder.control("".to_string())));
        controls.insert("email".to_string(), Rc::new(form_builder.control("".to_string())));

        let form = Rc::new(form_builder.group(controls));

        Self {
            form_builder,
            user_form: signal(Some(form)),
        }
    }
}
```

## Testing Components

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ferric::testing::*;

    #[component_test]
    async fn test_counter_increment() {
        let fixture = TestBed::create_component::<CounterComponent>();
        let component = fixture.component_instance();

        assert_eq!(component.count.get(), 0);

        component.increment();

        assert_eq!(component.count.get(), 1);
    }
}
```

## Examples

See the [examples directory](../examples/) for complete working examples:
- `todo-app` - TodoMVC implementation
- `reactivity-demo` - Reactivity showcase
- `ssr-demo` - Server-side rendering
- `dynamic-components` - Dynamic component creation

## Next Steps

- **[Templates](TEMPLATES_GUIDE.md)** - Learn template syntax
- **[Directives](DIRECTIVES_GUIDE.md)** - Create custom directives
- **[Services](SERVICES_GUIDE.md)** - Build injectable services
- **[Testing](TESTING_GUIDE.md)** - Test your components

## License

MIT

