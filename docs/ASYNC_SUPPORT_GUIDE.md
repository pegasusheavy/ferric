# Ferric Async Support Guide

Comprehensive guide to asynchronous programming with Promises in Ferric.

## Table of Contents

1. [Overview](#overview)
2. [Promise API](#promise-api)
3. [Async Lifecycle Hooks](#async-lifecycle-hooks)
4. [Spawning Tasks](#spawning-tasks)
5. [Timeouts & Delays](#timeouts--delays)
6. [Debouncing & Throttling](#debouncing--throttling)
7. [Async Queue & Scheduler](#async-queue--scheduler)
8. [Examples](#examples)

## Overview

Ferric provides comprehensive async support built on Rust futures with seamless JavaScript Promise interop for WASM. The framework includes:

- **Promise API** - High-level Promise abstraction
- **Async Lifecycle Hooks** - Async component initialization/cleanup
- **Task Spawning** - Execute futures with cancellation support
- **Timeout Utilities** - Delays, timeouts, and time-based operations
- **Debounce/Throttle** - Rate limiting for expensive operations
- **Async Queue** - Task queue management
- **Priority Scheduler** - Task scheduling with priorities

## Promise API

### Creating Promises

```rust
use ferric_core::async_support::prelude::*;

// Create a pending promise
let promise = Promise::<String>::new();

// Create a resolved promise
let promise = Promise::resolved("Hello".to_string());

// Create a rejected promise
let promise = Promise::rejected("Error".to_string());

// Create from JavaScript Promise
#[cfg(target_arch = "wasm32")]
let promise = Promise::from_js(js_promise);
```

### Using Promises

```rust
// Resolve or reject
promise.resolve("Success!".to_string());
promise.reject("Failed".to_string());

// Check state
match promise.state() {
    PromiseState::Pending => println!("Still waiting..."),
    PromiseState::Resolved(value) => println!("Got: {}", value),
    PromiseState::Rejected(error) => println!("Error: {}", error),
}

// Register callbacks
promise.on_change(|state| {
    match state {
        PromiseState::Resolved(value) => println!("Resolved: {}", value),
        _ => {}
    }
});

// Chain transformations
let doubled = promise.then(|value: i32| value * 2);

// Handle errors
let recovered = promise.catch(|error| {
    println!("Caught error: {}", error);
    0 // Return default value
});

// Await the promise
let result = promise.await;
```

### Promise Combinators

```rust
use ferric_core::async_support::promise::helpers;

// Wait for all promises
let promises = vec![fetch_user(), fetch_posts(), fetch_comments()];
let all_results = helpers::all(promises).await;

// Race multiple promises (first to settle wins)
let promises = vec![fetch_from_cache(), fetch_from_network()];
let first_result = helpers::race(promises).await;

// Delay/Sleep
let delayed = helpers::delay(1000); // 1 second
delayed.await;

// Wrap a Future in a Promise
let promise = helpers::from_future(async {
    Ok("Hello".to_string())
});
```

### Deferred Promises

```rust
// Create a deferred promise (can resolve later)
let deferred = Deferred::<String>::new();
let promise = deferred.promise();

// Later, resolve it
deferred.resolve("Done!".to_string());

// Or reject it
deferred.reject("Failed!".to_string());
```

## Async Lifecycle Hooks

### AsyncOnInit

Async initialization hook for loading data:

```rust
use ferric_core::lifecycle::async_hooks::*;

#[component(selector = "user-profile")]
struct UserProfile {
    user: Signal<Option<User>>,
    user_service: Arc<UserService>,
}

#[async_trait::async_trait(?Send)]
impl AsyncOnInit for UserProfile {
    async fn async_on_init(&self) {
        // Fetch user data asynchronously
        match self.user_service.get_current_user().await {
            Ok(user) => self.user.set(Some(user)),
            Err(e) => web_sys::console::error_1(&format!("Error: {}", e).into()),
        }
    }
}
```

### AsyncAfterViewInit

Async operations after view initialization:

```rust
#[async_trait::async_trait(?Send)]
impl AsyncAfterViewInit for MapComponent {
    async fn async_after_view_init(&self) {
        // Load map data after DOM is ready
        let map_data = self.load_map_data().await.unwrap();
        self.render_map(map_data);
    }
}
```

### AsyncOnDestroy

Async cleanup operations:

```rust
#[async_trait::async_trait(?Send)]
impl AsyncOnDestroy for WebSocketComponent {
    async fn async_on_destroy(&self) {
        // Close WebSocket connection gracefully
        self.ws_connection.close().await;

        // Save state to server
        self.save_state().await;
    }
}
```

### AsyncDataLoader

Dedicated data loading hook:

```rust
#[async_trait::async_trait(?Send)]
impl AsyncDataLoader for ProductList {
    type Data = Vec<Product>;

    async fn load_data(&self) -> Result<Self::Data, String> {
        self.product_service
            .fetch_products()
            .await
            .map_err(|e| e.to_string())
    }
}

// Use in component
async fn init_products(&self) {
    match AsyncLifecycleRunner::load_data(self).await {
        Ok(products) => self.products.set(products),
        Err(e) => self.error.set(Some(e)),
    }
}
```

## Spawning Tasks

### Basic Task Spawning

```rust
use ferric_core::async_support::spawn::*;

// Spawn a task
spawn_local_task(async {
    let data = fetch_data().await;
    process(data);
});

// Spawn with handle (for cancellation)
let handle = spawn_local_with_handle(async {
    loop {
        update().await;
        sleep(Duration::from_secs(1)).await;
    }
});

// Cancel the task
handle.cancel();
```

### Task with Result Callback

```rust
spawn_local_result(
    async { fetch_users().await },
    |users| {
        println!("Loaded {} users", users.len());
    }
);
```

## Timeouts & Delays

### Sleep/Delay

```rust
use ferric_core::async_support::timeout::*;

// Sleep for duration
sleep(Duration::from_millis(500)).await;

// Using helper
use ferric_core::async_support::promise::helpers;
helpers::delay(1000).await; // 1 second
```

### Timeout for Operations

```rust
use std::time::Duration;

// Add timeout to any future
let result = timeout(
    Duration::from_secs(5),
    fetch_data()
).await;

match result {
    Ok(data) => println!("Got data: {:?}", data),
    Err(TimeoutError) => println!("Operation timed out!"),
}
```

## Debouncing & Throttling

### Debouncing

Delay execution until after calls have stopped:

```rust
use ferric_core::async_support::debounce::*;

let search = debounce(
    move || {
        perform_search(query.get());
    },
    Duration::from_millis(300)
);

// Call it multiple times - only executes once after 300ms
search.call();
search.call();
search.call(); // Only this triggers execution

// Cancel pending calls
search.cancel();
```

### Throttling

Limit execution rate:

```rust
use ferric_core::async_support::throttle::*;

let update_handler = throttle(
    move || {
        update_ui();
    },
    Duration::from_millis(100)
);

// Calls are limited to once per 100ms
for _ in 0..1000 {
    update_handler.call(); // Only executes ~10 times
}
```

## Async Queue & Scheduler

### Async Queue

Manage task queues:

```rust
use ferric_core::async_support::queue::*;

let queue = AsyncQueue::new();

// Enqueue tasks
queue.enqueue(Task::FetchData);
queue.enqueue(Task::ProcessData);
queue.enqueue(Task::SaveData);

// Process queue
queue.process(|task| async move {
    match task {
        Task::FetchData => fetch().await,
        Task::ProcessData => process().await,
        Task::SaveData => save().await,
    }
}).await;
```

### Priority Scheduler

Schedule tasks with priorities:

```rust
use ferric_core::async_support::scheduler::*;

let scheduler = AsyncScheduler::new();

// Schedule with different priorities
scheduler.schedule_with_priority(
    async { critical_update().await },
    Priority::Critical
);

scheduler.schedule_with_priority(
    async { background_sync().await },
    Priority::Low
);

scheduler.schedule(async {
    normal_task().await
}); // Default: Normal priority

// Run all tasks (critical first, then high, normal, low)
scheduler.run().await;

// Use global scheduler
schedule_high_priority(async {
    urgent_task().await
});
```

## Examples

### Complete Async Component

```rust
use ferric_core::prelude::*;
use ferric_core::async_support::prelude::*;

#[component(
    selector = "user-dashboard",
    template = r#"
        <div class="dashboard">
            <h1>Welcome, {{ user_name }}</h1>
            <div *if="loading">Loading...</div>
            <div *if="error">Error: {{ error }}</div>
            <div *if="data">
                <user-stats [stats]="stats"></user-stats>
            </div>
        </div>
    "#
)]
struct UserDashboard {
    user_service: Arc<UserService>,
    user_name: Signal<String>,
    stats: Signal<Option<UserStats>>,
    loading: Signal<bool>,
    error: Signal<Option<String>>,
}

#[async_trait::async_trait(?Send)]
impl AsyncOnInit for UserDashboard {
    async fn async_on_init(&self) {
        self.loading.set(true);
        self.error.set(None);

        // Load user data with timeout
        let result = timeout(
            Duration::from_secs(10),
            self.user_service.get_user_stats()
        ).await;

        match result {
            Ok(Ok(stats)) => {
                self.stats.set(Some(stats.clone()));
                self.user_name.set(stats.name);
            }
            Ok(Err(e)) => {
                self.error.set(Some(e.to_string()));
            }
            Err(_) => {
                self.error.set(Some("Request timed out".to_string()));
            }
        }

        self.loading.set(false);
    }
}

#[async_trait::async_trait(?Send)]
impl AsyncOnDestroy for UserDashboard {
    async fn async_on_destroy(&self) {
        // Save current state
        if let Some(stats) = self.stats.get() {
            let _ = self.user_service.save_stats(&stats).await;
        }
    }
}
```

### Search with Debouncing

```rust
#[component(selector = "search-box")]
struct SearchBox {
    query: Signal<String>,
    results: Signal<Vec<SearchResult>>,
    search_service: Arc<SearchService>,
    debounced_search: Debounced<Box<dyn Fn()>>,
}

impl SearchBox {
    fn new() -> Self {
        let query = signal!("");
        let results = signal!(vec![]);
        let search_service = inject!(SearchService);

        let query_clone = query.clone();
        let results_clone = results.clone();
        let service_clone = search_service.clone();

        let debounced_search = debounce(
            Box::new(move || {
                let q = query_clone.get();
                let r = results_clone.clone();
                let s = service_clone.clone();

                spawn_local_task(async move {
                    match s.search(&q).await {
                        Ok(res) => r.set(res),
                        Err(e) => web_sys::console::error_1(&e.to_string().into()),
                    }
                });
            }),
            Duration::from_millis(300)
        );

        Self {
            query,
            results,
            search_service,
            debounced_search,
        }
    }

    fn on_input_change(&self, value: String) {
        self.query.set(value);
        self.debounced_search.call();
    }
}
```

### Promise-based HTTP Request

```rust
async fn fetch_user_data(user_id: u32) -> Result<User, String> {
    // Create a promise-based HTTP request
    let promise = Promise::new();
    let promise_clone = promise.clone();

    spawn_local_task(async move {
        match http_get(&format!("/api/users/{}", user_id)).await {
            Ok(user) => promise_clone.resolve(user),
            Err(e) => promise_clone.reject(e.to_string()),
        }
    });

    promise.await
}

// Use with combinators
async fn load_user_profile(user_id: u32) {
    let promises = vec![
        fetch_user_data(user_id),
        fetch_user_posts(user_id),
        fetch_user_friends(user_id),
    ];

    match helpers::all(promises).await {
        Ok(results) => {
            let (user, posts, friends) = (results[0], results[1], results[2]);
            render_profile(user, posts, friends);
        }
        Err(e) => {
            show_error(&e);
        }
    }
}
```

## Best Practices

1. **Use Async Lifecycle Hooks** - Leverage `AsyncOnInit` for data loading
2. **Debounce User Input** - Use `debounce` for search, validation, etc.
3. **Add Timeouts** - Always timeout network requests
4. **Handle Errors** - Use `Promise.catch()` or match on `Result`
5. **Cancel Tasks** - Use `TaskHandle` to cancel long-running operations
6. **Priority Scheduling** - Use `Priority::High` for user-facing tasks
7. **Queue Background Work** - Use `AsyncQueue` for non-urgent tasks
8. **Throttle Updates** - Use `throttle` for high-frequency events (scroll, resize)

## Performance Tips

- Use `Promise::all()` for parallel operations
- Debounce expensive operations (API calls, validations)
- Throttle high-frequency events
- Schedule background tasks with low priority
- Cancel unnecessary async operations on component destroy
- Use `Promise::race()` for cache-or-network patterns

## Migration from Sync Code

### Before (Synchronous)

```rust
impl OnInit for MyComponent {
    fn on_init(&mut self) {
        // Blocking call (not recommended in WASM)
        let data = fetch_data_sync();
        self.data.set(data);
    }
}
```

### After (Asynchronous)

```rust
#[async_trait::async_trait(?Send)]
impl AsyncOnInit for MyComponent {
    async fn async_on_init(&self) {
        // Non-blocking async call
        match fetch_data().await {
            Ok(data) => self.data.set(data),
            Err(e) => self.error.set(Some(e)),
        }
    }
}
```

## See Also

- [Async Pipe Documentation](ferric-core/src/pipes/async_pipe.rs)
- [Async Validators](ferric-forms/src/validators/async_validator.rs)
- [HTTP Client](ferric-http/)
- [IndexedDB](ferric-idb/)

---

**Ferric now provides comprehensive async/Promise support for building modern, responsive web applications!** 🚀

