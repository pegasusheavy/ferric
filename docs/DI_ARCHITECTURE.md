# Ferric Dependency Injection Architecture

**Making DI the backbone of the framework**

## Overview

Ferric's Dependency Injection system is central to its architecture, following Angular's philosophy where nearly every framework feature leverages DI for configuration, services, and extensibility. This document explains how DI is deeply integrated throughout the framework.

## Core Principles

1. **Services First**: All major framework features (Router, HTTP, Forms, etc.) are provided as injectable services
2. **Configuration via Tokens**: Use injection tokens for configuration values instead of global constants
3. **Hierarchical Injection**: Components create child injectors for scoped dependencies
4. **Module-Based Organization**: Organize services into cohesive modules (HttpModule, RouterModule, etc.)
5. **Type Safety**: Leverage Rust's type system for compile-time dependency verification

## Framework Systems Using DI

### 1. HTTP Client (`ferric-http`)

The HTTP client is fully injectable with configuration through DI tokens:

```rust
use ferric_http::{HttpModule, HttpClient, http_tokens};
use ferric::di::Injector;

// Configure HTTP client via DI
let injector = Injector::root();

// Set base URL
injector.register_token(&http_tokens::HTTP_BASE_URL, "https://api.example.com".to_string());

// Set timeout
injector.register_token(&http_tokens::HTTP_TIMEOUT_MS, 30000);

// Set bearer token
injector.register_token(&http_tokens::HTTP_BEARER_TOKEN, "your-token-here".to_string());

// Register the client
HttpModule::provide_with_base_url(&injector, "https://api.example.com");

// Inject in your service
struct UserService {
    http: Rc<HttpClient>,
}

impl Injectable for UserService {
    fn create(injector: &Injector) -> Self {
        Self {
            http: injector.resolve_required::<HttpClient>(),
        }
    }
}
```

**Available HTTP Tokens:**
- `HTTP_BASE_URL` - Base URL for all requests
- `HTTP_TIMEOUT_MS` - Default timeout in milliseconds
- `HTTP_CREDENTIALS` - Credentials mode (SameOrigin, Include, Omit)
- `HTTP_BEARER_TOKEN` - Bearer token for authentication
- `HTTP_API_KEY` - API key for authentication
- `HTTP_DEFAULT_HEADERS` - Default headers for all requests

### 2. Router (`ferric-core/router`)

The Router is injectable and configured through DI:

```rust
use ferric::router::{RouterModule, Router, Route, router_tokens};

// Configure router
let injector = Injector::root();

// Set base href
injector.register_token(&router_tokens::ROUTER_BASE_HREF, "/app".to_string());

// Register routes
let routes = vec![
    Route::new("/").component("app-home"),
    Route::new("/users").component("app-users"),
];
injector.register_token(&router_tokens::ROUTER_ROUTES, routes);

// Provide router
RouterModule::provide_with_routes(&injector, routes);

// Guards and Resolvers are also injectable
struct AuthGuard {
    auth: Rc<AuthService>,
}

impl Injectable for AuthGuard {
    fn create(injector: &Injector) -> Self {
        Self {
            auth: injector.resolve_required::<AuthService>(),
        }
    }
}

impl CanActivate for AuthGuard {
    fn can_activate(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult {
        if self.auth.is_authenticated() {
            GuardResult::Allow
        } else {
            GuardResult::Deny("/login")
        }
    }
}
```

**Available Router Tokens:**
- `ROUTER_BASE_HREF` - Base href for the application
- `ROUTER_USE_HASH` - Enable hash-based routing
- `ROUTER_SCROLL_RESTORATION` - Scroll restoration strategy
- `ROUTER_ENABLE_TRACING` - Enable route tracing for debugging
- `ROUTER_INITIAL_NAVIGATION` - Initial navigation URL
- `ROUTER_ROUTES` - Routes configuration

### 3. Forms (`ferric-forms`)

Forms provide a FormBuilder service through DI:

```rust
use ferric_forms::{FormsModule, FormBuilder, form_tokens};

// Setup forms module
let injector = Injector::root();
FormsModule::provide(&injector);

// Inject FormBuilder in your component
#[component(selector = "user-form")]
struct UserFormComponent {
    fb: Rc<FormBuilder>,
    form: FormGroup,
}

impl Injectable for UserFormComponent {
    fn create(injector: &Injector) -> Self {
        let fb = injector.resolve_required::<FormBuilder>();

        let form = fb.group();
        form.add_control("name", fb.control("".to_string()));
        form.add_control("email", fb.control("".to_string()));

        Self { fb, form }
    }
}
```

**Available Forms Tokens:**
- `FORM_GLOBAL_VALIDATORS` - Global validators for all forms
- `FORM_GLOBAL_ASYNC_VALIDATORS` - Global async validators
- `FORM_DEFAULT_UPDATE_ON` - Default update strategy
- `FORM_ENABLE_PERSISTENCE` - Enable form state persistence

### 4. Pipes (`ferric-core/pipes`)

Pipes can be registered and retrieved through DI:

```rust
use ferric::pipes::{PipesModule, InjectablePipeRegistry};

// Setup pipes module
let injector = Injector::root();
PipesModule::provide(&injector);

// Custom pipe
struct CustomPipe;

impl Pipe for CustomPipe {
    fn name(&self) -> &'static str { "custom" }
    fn transform(&self, value: &str, _args: &PipeArgs) -> String {
        value.to_uppercase()
    }
}

// Register custom pipe
let registry = injector.resolve_required::<InjectablePipeRegistry>();
registry.register("custom", Rc::new(CustomPipe));
```

### 5. Framework Configuration

Core framework settings are configured via DI:

```rust
use ferric::{CoreModule, FrameworkConfig, config_tokens};

// Development configuration
let injector = Injector::root();
CoreModule::provide_for_development(&injector);

// Access configuration in services
struct MyService {
    config: Rc<FrameworkConfig>,
}

impl Injectable for MyService {
    fn create(injector: &Injector) -> Self {
        Self {
            config: injector.resolve_required::<FrameworkConfig>(),
        }
    }
}

// Production configuration
let prod_injector = Injector::root();
CoreModule::provide_for_production(&prod_injector);
```

**Available Config Tokens:**
- `DEBUG_CHANGE_DETECTION` - Enable change detection debugging
- `DEBUG_LIFECYCLE` - Enable lifecycle hook debugging
- `DEBUG_DI` - Enable DI resolution debugging
- `PRODUCTION_MODE` - Production mode flag
- `APP_NAME` - Application name
- `APP_VERSION` - Application version
- `ENVIRONMENT` - Environment name (development, production, etc.)
- `LOCALE` - Locale for internationalization
- `TIMEZONE` - Timezone for date/time formatting

## Creating Injectable Services

### Basic Injectable Service

```rust
use ferric::di::{Injectable, Injector};
use std::rc::Rc;

pub struct LoggingService {
    level: LogLevel,
}

impl Injectable for LoggingService {
    fn create(injector: &Injector) -> Self {
        // Services can depend on other services
        let config = injector.resolve::<FrameworkConfig>();

        Self {
            level: if config.is_some() && config.unwrap().is_production() {
                LogLevel::Error
            } else {
                LogLevel::Debug
            },
        }
    }
}
```

### Service with Dependencies

```rust
pub struct UserService {
    http: Rc<HttpClient>,
    logger: Rc<LoggingService>,
}

impl Injectable for UserService {
    fn create(injector: &Injector) -> Self {
        Self {
            http: injector.resolve_required::<HttpClient>(),
            logger: injector.resolve_required::<LoggingService>(),
        }
    }
}
```

### Service with Token Configuration

```rust
const API_ENDPOINT: InjectionToken<String> =
    InjectionToken::with_id("API_ENDPOINT", 10001);

pub struct DataService {
    http: Rc<HttpClient>,
    endpoint: String,
}

impl Injectable for DataService {
    fn create(injector: &Injector) -> Self {
        Self {
            http: injector.resolve_required::<HttpClient>(),
            endpoint: injector
                .resolve_token(&API_ENDPOINT)
                .map(|s| (*s).clone())
                .unwrap_or_else(|| "/api/data".to_string()),
        }
    }
}
```

## Module Pattern

Organize related services into modules:

```rust
pub struct MyFeatureModule;

impl MyFeatureModule {
    pub fn provide(injector: &Injector) {
        // Register services
        injector.register_singleton::<FeatureService>();
        injector.register_singleton::<HelperService>();

        // Register tokens
        injector.register_token(&FEATURE_CONFIG, FeatureConfig::default());

        // Register multi-providers
        injector.add_multi(&VALIDATORS, Box::new(CustomValidator));
    }

    pub fn provide_for_root(injector: &Injector) {
        Self::provide(injector);
        // Additional root-level configuration
        injector.register_singleton::<GlobalFeatureService>();
    }
}
```

## Component DI Integration

Components automatically get their own child injector:

```rust
#[component(
    selector = "user-profile",
    providers = [UserService, ProfileService]
)]
struct UserProfileComponent {
    user_service: Rc<UserService>,
    profile_service: Rc<ProfileService>,
}

impl Injectable for UserProfileComponent {
    fn create(injector: &Injector) -> Self {
        // Injector here is the component's child injector
        // It inherits from parent but has component-specific services
        Self {
            user_service: injector.resolve_required::<UserService>(),
            profile_service: injector.resolve_required::<ProfileService>(),
        }
    }
}
```

## Best Practices

### 1. Use Injection Tokens for Configuration

**❌ Bad:**
```rust
const API_URL: &str = "https://api.example.com";

struct MyService {
    url: &'static str,
}
```

**✅ Good:**
```rust
const API_URL: InjectionToken<String> = InjectionToken::with_id("API_URL", 1001);

struct MyService {
    url: String,
}

impl Injectable for MyService {
    fn create(injector: &Injector) -> Self {
        Self {
            url: injector.resolve_token_required(&API_URL).to_string(),
        }
    }
}
```

### 2. Prefer Constructor Injection

**✅ Good:**
```rust
impl Injectable for UserService {
    fn create(injector: &Injector) -> Self {
        Self {
            http: injector.resolve_required::<HttpClient>(),
            logger: injector.resolve_required::<LoggingService>(),
        }
    }
}
```

### 3. Use Modules for Organization

Group related services and configuration:

```rust
pub struct AuthModule;

impl AuthModule {
    pub fn provide(injector: &Injector) {
        // Auth services
        injector.register_singleton::<AuthService>();
        injector.register_singleton::<TokenService>();

        // Auth guards
        injector.register_singleton::<AuthGuard>();
        injector.register_singleton::<RoleGuard>();

        // Auth configuration
        injector.register_token(&AUTH_API_URL, "https://auth.example.com".to_string());
    }
}
```

### 4. Leverage Hierarchical Injection

Use child injectors for scoping:

```rust
// Root injector - shared across app
let root = Injector::root();
HttpModule::provide(&root);
RouterModule::provide(&root);

// Feature module injector - scoped to feature
let feature_injector = Injector::child(Rc::new(root));
FeatureModule::provide(&feature_injector);

// Component injector - scoped to component instance
let component_injector = Injector::child(Rc::new(feature_injector));
component_injector.register_singleton::<ComponentService>();
```

### 5. Use Multi-Providers for Extensibility

```rust
const INTERCEPTORS: MultiToken<Rc<dyn HttpInterceptor>> =
    MultiToken::with_id("HTTP_INTERCEPTORS", 2001);

// Register multiple interceptors
injector.add_multi(&INTERCEPTORS, Rc::new(LoggingInterceptor));
injector.add_multi(&INTERCEPTORS, Rc::new(AuthInterceptor));
injector.add_multi(&INTERCEPTORS, Rc::new(RetryInterceptor));

// Resolve all
let interceptors = injector.resolve_multi(&INTERCEPTORS);
```

## Testing with DI

DI makes testing straightforward:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockHttpClient;

    impl Injectable for MockHttpClient {
        fn create(_: &Injector) -> Self {
            MockHttpClient
        }
    }

    #[test]
    fn test_user_service() {
        let injector = Injector::root();

        // Register mock
        injector.register_singleton::<MockHttpClient>();

        // Test service with mock dependency
        injector.register_factory(
            |inj| UserService {
                http: inj.resolve_required::<MockHttpClient>(),
            },
            Scope::Transient
        );

        let service = injector.resolve::<UserService>().unwrap();
        // Test service...
    }
}
```

## Performance Considerations

1. **Singleton Caching**: Services registered as singletons are cached, ensuring O(1) lookups after first resolution
2. **Lazy Resolution**: Services are only created when first requested
3. **Rc-based Sharing**: Uses `Rc<T>` for zero-copy service sharing
4. **FxHashMap**: Fast hashing for TypeId-based lookups
5. **Compile-time Type Safety**: No runtime type checking overhead after resolution

## Summary

Ferric's DI system is the foundation that ties all framework features together:

- ✅ **HTTP Client**: Fully injectable with token-based configuration
- ✅ **Router**: Injectable with guards and resolvers using DI
- ✅ **Forms**: FormBuilder service provided through DI
- ✅ **Pipes**: Registry service for managing custom pipes
- ✅ **Framework Config**: Core settings via injection tokens
- ✅ **Components**: Automatic child injector creation
- ✅ **Testing**: Easy mocking and dependency replacement

By making DI central to the framework, Ferric achieves:
- **Modularity**: Features are loosely coupled and easily replaceable
- **Testability**: Dependencies can be mocked and controlled
- **Configuration**: Everything configurable via injection tokens
- **Extensibility**: Easy to add custom services and providers
- **Type Safety**: Compile-time verification of dependencies

This architecture ensures that DI is not just a feature, but the backbone of how developers build Ferric applications.

