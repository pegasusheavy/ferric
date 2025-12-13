//! Example demonstrating route activation with guards and resolvers.

use ferric_core::router::{
    Route, Router, CanActivate, GuardResult, RouterStateSnapshot,
    Resolve, ResolveResult, ActivatedRouteSnapshot,
};
use wasm_bindgen::prelude::*;

// ============================================================================
// Guards
// ============================================================================

/// Authentication guard that checks if user is logged in
struct AuthGuard;

impl CanActivate for AuthGuard {
    fn can_activate(&self, _route: &Route, _state: &RouterStateSnapshot) -> GuardResult {
        // In a real app, this would check authentication status
        let is_authenticated = true; // Simulate authenticated user

        if is_authenticated {
            GuardResult::Allow
        } else {
            GuardResult::Redirect("/login".to_string())
        }
    }
}

/// Admin guard that checks if user has admin role
struct AdminGuard;

impl CanActivate for AdminGuard {
    fn can_activate(&self, _route: &Route, _state: &RouterStateSnapshot) -> GuardResult {
        let is_admin = false; // Simulate non-admin user

        if is_admin {
            GuardResult::Allow
        } else {
            GuardResult::Deny
        }
    }
}

// ============================================================================
// Resolvers
// ============================================================================

/// User resolver that fetches user data before route activation
struct UserResolver;

impl Resolve for UserResolver {
    type Output = String;

    fn resolve(&self, route: &ActivatedRouteSnapshot) -> ResolveResult<Self::Output> {
        // Extract user ID from route params
        if let Some(user_id) = route.params.get("id") {
            // Simulate fetching user data
            ResolveResult::Ready(format!("User #{}", user_id))
        } else {
            ResolveResult::Error("No user ID provided".to_string())
        }
    }
}

/// Posts resolver that fetches posts data
struct PostsResolver;

impl Resolve for PostsResolver {
    type Output = Vec<String>;

    fn resolve(&self, _route: &ActivatedRouteSnapshot) -> ResolveResult<Self::Output> {
        // Simulate fetching posts
        ResolveResult::Ready(vec![
            "Post 1".to_string(),
            "Post 2".to_string(),
            "Post 3".to_string(),
        ])
    }
}

// ============================================================================
// Setup and Examples
// ============================================================================

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set up console error panic hook
    console_error_panic_hook::set_once();

    // Configure routes with guards and resolvers
    let routes = vec![
        Route::new("/")
            .component("app-home"),

        Route::new("/login")
            .component("app-login"),

        Route::new("/profile")
            .component("app-profile")
            .can_activate("auth"), // Requires authentication

        Route::new("/users/:id")
            .component("app-user-detail")
            .can_activate("auth")
            .resolve("user", "userResolver"), // Pre-fetch user data

        Route::new("/admin")
            .component("app-admin")
            .can_activate("auth")
            .can_activate("admin") // Requires both auth and admin
            .children(vec![
                Route::new("dashboard").component("admin-dashboard"),
                Route::new("users").component("admin-users"),
            ]),

        Route::new("/posts")
            .component("app-posts")
            .resolve("posts", "postsResolver"), // Pre-fetch posts
    ];

    // Create router
    let router = Router::new(routes);

    // Register guards
    router.register_guard("auth", AuthGuard);
    router.register_guard("admin", AdminGuard);

    // Register resolvers
    router.register_resolver("userResolver", UserResolver);
    router.register_resolver("postsResolver", PostsResolver);

    // Subscribe to navigation events
    router.events().subscribe(|event| {
        web_sys::console::log_1(&format!("Router event: {:?}", event).into());
    });

    // Initialize router
    router.init()?;

    web_sys::console::log_1(&"Router example initialized!".into());
    Ok(())
}

