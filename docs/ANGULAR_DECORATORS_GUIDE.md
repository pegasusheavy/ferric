# Angular-Like Decorators in Ferric

Complete guide to all Angular-style decorators available in the Ferric framework.

## Table of Contents

1. [Component & Module Organization](#component--module-organization)
2. [Directives & Pipes](#directives--pipes)
3. [Dependency Injection](#dependency-injection)
4. [Component Communication](#component-communication)
5. [Host Interactions](#host-interactions)
6. [View Queries](#view-queries)
7. [Routing](#routing)
8. [HTTP](#http)

---

## Component & Module Organization

### `#[component]`

Define a component with metadata (equivalent to Angular's `@Component`).

```rust
#[component(
    selector = "app-user-card",
    template = r#"
        <div class="card">
            <h2>{{ name }}</h2>
            <p>{{ email }}</p>
        </div>
    "#,
    styles = ".card { padding: 20px; }",
    change_detection = "OnPush"
)]
pub struct UserCardComponent {
    #[input]
    name: String,

    #[input]
    email: String,
}
```

**Attributes:**
- `selector` (required) - CSS selector for the component
- `template` or `template_url` - Component template
- `styles` or `style_urls` - Component styles
- `encapsulation` - View encapsulation (`Emulated`, `ShadowDom`, `None`)
- `change_detection` - Change detection strategy (`Default`, `OnPush`)

### `#[module]`

Define a module to organize application structure (equivalent to Angular's `@NgModule`).

```rust
#[module]
pub struct AppModule;

impl AppModule {
    fn imports(&self) -> Vec<Box<dyn Module>> {
        vec![
            Box::new(CommonModule),
            Box::new(FormsModule),
        ]
    }

    fn declarations(&self) -> Vec<Box<dyn Component>> {
        vec![
            Box::new(AppComponent::default()),
            Box::new(HeaderComponent::default()),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(UserService::provider()),
        ]
    }
}
```

**Use Cases:**
- Organize related components, directives, and pipes
- Configure dependency injection
- Import shared functionality
- Define module boundaries

---

## Directives & Pipes

### `#[directive]`

Create a directive to add behavior to elements (equivalent to Angular's `@Directive`).

```rust
#[directive]
pub struct HighlightDirective {
    #[input]
    color: String,

    element: ElementRef,
}

impl HighlightDirective {
    pub fn on_init(&self) {
        self.element.set_style("background-color", &self.color);
    }

    #[host_listener("mouseenter")]
    pub fn on_mouse_enter(&self) {
        self.element.set_style("background-color", "yellow");
    }

    #[host_listener("mouseleave")]
    pub fn on_mouse_leave(&self) {
        self.element.set_style("background-color", &self.color);
    }
}
```

**Usage in templates:**
```html
<p [appHighlight]="'blue'">Hover over me!</p>
```

### `#[pipe]`

Create a pipe for data transformation (equivalent to Angular's `@Pipe`).

```rust
#[pipe]
pub struct UpperCasePipe;

impl UpperCasePipe {
    pub fn transform(&self, value: String) -> String {
        value.to_uppercase()
    }
}

#[pipe]
pub struct DatePipe;

impl DatePipe {
    pub fn transform(&self, value: i64, format: Option<&str>) -> String {
        let format = format.unwrap_or("%Y-%m-%d");
        // Format date logic
        format!("2024-01-01") // Simplified
    }
}
```

**Usage in templates:**
```html
<p>{{ name | uppercase }}</p>
<p>{{ timestamp | date:"Y-m-d H:i:s" }}</p>
```

---

## Dependency Injection

### `#[injectable]`

Mark a service as injectable (equivalent to Angular's `@Injectable`).

```rust
#[injectable]
pub struct UserService {
    http: Arc<HttpClient>,
    users: RefCell<Vec<User>>,
}

impl UserService {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self {
            http,
            users: RefCell::new(vec![]),
        }
    }

    pub async fn fetch_users(&self) -> Result<Vec<User>, Error> {
        let users = self.http
            .get("/api/users")
            .send()
            .await?
            .json()
            .await?;

        *self.users.borrow_mut() = users.clone();
        Ok(users)
    }
}
```

**Usage in components:**
```rust
#[component(selector = "user-list")]
pub struct UserListComponent {
    user_service: Arc<UserService>,
}

impl UserListComponent {
    pub fn new() -> Self {
        Self {
            user_service: inject!(UserService),
        }
    }
}
```

---

## Component Communication

### `#[input]`

Define input properties that receive data from parent components.

```rust
#[component(selector = "child-component")]
pub struct ChildComponent {
    #[input]
    user_name: String,

    #[input(alias = "userId", required)]
    id: u32,

    #[input(transform = "parse_date")]
    birthday: Date,
}
```

**Parent template:**
```html
<child-component
    [userName]="currentUser.name"
    [userId]="currentUser.id"
    [birthday]="currentUser.dob">
</child-component>
```

### `#[output]`

Define output event emitters to send data to parent components.

```rust
#[component(selector = "button-component")]
pub struct ButtonComponent {
    #[output]
    clicked: EventEmitter<()>,

    #[output(alias = "valueChange")]
    value_changed: EventEmitter<String>,
}

impl ButtonComponent {
    pub fn on_click(&self) {
        emit!(self.clicked, ());
    }

    pub fn on_value_change(&self, new_value: String) {
        emit!(self.value_changed, new_value);
    }
}
```

**Parent template:**
```html
<button-component
    (clicked)="handleClick()"
    (valueChange)="onValueChange($event)">
</button-component>
```

---

## Host Interactions

### `#[host_listener]`

Listen to events on the host element.

```rust
#[component(selector = "click-tracker")]
pub struct ClickTrackerComponent {
    click_count: Signal<i32>,
}

impl ClickTrackerComponent {
    #[host_listener("click")]
    pub fn on_click(&self) {
        self.click_count.update(|n| n + 1);
    }

    #[host_listener("keydown.enter")]
    pub fn on_enter(&self, event: KeyboardEvent) {
        web_sys::console::log_1(&"Enter pressed".into());
    }

    #[host_listener("window:resize")]
    pub fn on_window_resize(&self) {
        web_sys::console::log_1(&"Window resized".into());
    }
}
```

### `#[host_binding]`

Bind properties to the host element.

```rust
#[component(selector = "themed-box")]
pub struct ThemedBoxComponent {
    #[host_binding("class.active")]
    is_active: Signal<bool>,

    #[host_binding("attr.role")]
    role: String,

    #[host_binding("style.color")]
    text_color: Signal<String>,
}
```

---

## View Queries

### `#[view_child]` / `#[view_children]`

Query child elements in the component's view.

```rust
#[component(selector = "parent-component")]
pub struct ParentComponent {
    #[view_child("myInput")]
    input_ref: Option<ElementRef>,

    #[view_children("item")]
    items: Vec<ElementRef>,
}

impl ParentComponent {
    pub fn focus_input(&self) {
        if let Some(input) = &self.input_ref {
            input.focus();
        }
    }

    pub fn count_items(&self) -> usize {
        self.items.len()
    }
}
```

**Template:**
```html
<input #myInput type="text" />
<div #item *for="let item of items">{{ item }}</div>
```

---

## Routing

### `#[guard]`

Define route guards to control navigation (equivalent to Angular's `CanActivate`, `CanDeactivate`, etc.).

```rust
#[guard]
pub struct AuthGuard {
    auth_service: Arc<AuthService>,
}

impl AuthGuard {
    pub fn check(
        &self,
        route: &ActivatedRouteSnapshot,
        state: &RouterStateSnapshot,
    ) -> GuardResult {
        if self.auth_service.is_authenticated() {
            GuardResult::Allow
        } else {
            GuardResult::Redirect("/login".to_string())
        }
    }
}

// Different guard types
#[guard] // Defaults to CanActivate
pub struct AdminGuard;

#[guard] // For CanDeactivate
pub struct UnsavedChangesGuard;

impl UnsavedChangesGuard {
    pub fn check_deactivate(
        &self,
        component: &dyn Any,
        current_route: &ActivatedRouteSnapshot,
        current_state: &RouterStateSnapshot,
        next_state: &RouterStateSnapshot,
    ) -> GuardResult {
        // Check if component has unsaved changes
        GuardResult::Allow
    }
}
```

**Route configuration:**
```rust
Route::new()
    .path("/admin")
    .component("admin-panel")
    .can_activate(vec!["AuthGuard", "AdminGuard"])
```

### `#[resolver]`

Pre-fetch data before activating a route (equivalent to Angular's `Resolve`).

```rust
#[resolver]
pub struct UserResolver {
    user_service: Arc<UserService>,
}

impl UserResolver {
    pub fn load_data(&self, route: &ActivatedRouteSnapshot) -> ResolverResult {
        let user_id = route.params.get("id").unwrap();

        match self.user_service.get_user(user_id) {
            Ok(user) => ResolverResult::Success(Box::new(user)),
            Err(e) => ResolverResult::Error(e.to_string()),
        }
    }
}
```

**Route configuration:**
```rust
Route::new()
    .path("/user/:id")
    .component("user-profile")
    .resolve(hashmap! { "user" => "UserResolver" })
```

---

## HTTP

### `#[interceptor]`

Intercept and modify HTTP requests/responses (equivalent to Angular's `HttpInterceptor`).

```rust
#[interceptor]
pub struct AuthInterceptor {
    token_service: Arc<TokenService>,
}

impl AuthInterceptor {
    pub fn handle(
        &self,
        mut request: Request,
        next: Next,
    ) -> ResponseFuture {
        // Add authentication token
        if let Some(token) = self.token_service.get_token() {
            request.headers_mut().insert(
                "Authorization",
                format!("Bearer {}", token)
            );
        }

        Box::pin(async move {
            let response = next(request).await?;

            // Log response
            web_sys::console::log_1(&format!("Status: {}", response.status()).into());

            Ok(response)
        })
    }
}

#[interceptor]
pub struct LoggingInterceptor;

impl LoggingInterceptor {
    pub fn handle(&self, request: Request, next: Next) -> ResponseFuture {
        let start = web_sys::window().unwrap().performance().unwrap().now();
        let url = request.url().to_string();

        Box::pin(async move {
            let response = next(request).await?;
            let duration = web_sys::window().unwrap().performance().unwrap().now() - start;

            web_sys::console::log_1(
                &format!("{} completed in {:.2}ms", url, duration).into()
            );

            Ok(response)
        })
    }
}
```

---

## Complete Example

Here's a complete example using multiple decorators together:

```rust
// Module definition
#[module]
pub struct UserModule;

impl UserModule {
    fn declarations(&self) -> Vec<Box<dyn Component>> {
        vec![
            Box::new(UserListComponent::default()),
            Box::new(UserCardComponent::default()),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(UserService::provider()),
        ]
    }
}

// Service
#[injectable]
pub struct UserService {
    http: Arc<HttpClient>,
}

// Guard
#[guard]
pub struct UserGuard {
    user_service: Arc<UserService>,
}

// Resolver
#[resolver]
pub struct UsersResolver {
    user_service: Arc<UserService>,
}

// Pipe
#[pipe]
pub struct UserRolePipe;

// Directive
#[directive]
pub struct UserTooltipDirective {
    #[input]
    user: User,
}

// Component
#[component(
    selector = "user-list",
    template = r#"
        <div class="users">
            <user-card
                *for="let user of users"
                [user]="user"
                [appUserTooltip]="user"
                (selected)="onUserSelected($event)">
            </user-card>
        </div>
    "#
)]
pub struct UserListComponent {
    users: Signal<Vec<User>>,
    user_service: Arc<UserService>,
}

#[component(
    selector = "user-card",
    template = r#"
        <div class="card" (click)="select()">
            <h3>{{ user.name }}</h3>
            <p>{{ user.role | userRole }}</p>
        </div>
    "#
)]
pub struct UserCardComponent {
    #[input]
    user: User,

    #[output]
    selected: EventEmitter<User>,

    pub fn select(&self) {
        emit!(self.selected, self.user.clone());
    }
}
```

---

## Comparison with Angular

| Angular | Ferric | Description |
|---------|--------|-------------|
| `@Component` | `#[component]` | Define components |
| `@NgModule` | `#[module]` | Organize modules |
| `@Directive` | `#[directive]` | Create directives |
| `@Pipe` | `#[pipe]` | Create pipes |
| `@Injectable` | `#[injectable]` | Mark services |
| `@Input` | `#[input]` | Input properties |
| `@Output` | `#[output]` | Output events |
| `@HostListener` | `#[host_listener]` | Host event listeners |
| `@HostBinding` | `#[host_binding]` | Host property bindings |
| `@ViewChild` | `#[view_child]` | Query single child |
| `@ViewChildren` | `#[view_children]` | Query multiple children |
| `CanActivate` | `#[guard]` | Route guards |
| `Resolve` | `#[resolver]` | Route resolvers |
| `HttpInterceptor` | `#[interceptor]` | HTTP interceptors |

---

## Best Practices

1. **Use decorators consistently** - Follow Angular patterns for familiar DX
2. **Organize with modules** - Group related components, directives, and services
3. **Leverage DI** - Use `#[injectable]` and `inject!()` for loose coupling
4. **Guard your routes** - Implement proper authentication and authorization
5. **Intercept HTTP** - Add common headers, logging, and error handling
6. **Transform data with pipes** - Keep templates clean and reusable
7. **Directive for behavior** - Add reusable behaviors without templates
8. **Resolve data early** - Pre-fetch data with resolvers for better UX

---

## Next Steps

- See [MACROS_GUIDE.md](ferric-macros/MACROS_GUIDE.md) for utility macros
- See [FORMS_MACROS_GUIDE.md](ferric-forms-macros/FORMS_MACROS_GUIDE.md) for forms
- Check examples in `examples/` directory

---

**🎉 Ferric now provides a complete Angular-like decorator system for Rust!**

