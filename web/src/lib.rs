//! Ferric Documentation Website
//!
//! Built with Ferric framework to demonstrate its capabilities

use ferric_core::prelude::*;
use wasm_bindgen::prelude::*;

mod components;
mod pages;
mod services;

use components::*;
use pages::*;

/// Main application component
#[component(selector = "app-root")]
pub struct AppComponent {
    #[input]
    title: String,
}

impl Component for AppComponent {
    fn new() -> Self {
        Self {
            title: "Ferric Framework".to_string(),
        }
    }

    fn on_init(&mut self) {
        console_log!("🦀 Ferric Documentation Site Initialized!");
    }

    fn render(&self) -> Html {
        html! {
            <div class="min-h-screen bg-gray-50">
                <router-outlet />
            </div>
        }
    }
}

/// Application routes
pub fn get_routes() -> Vec<Route> {
    vec![
        Route::new("/", HomePage::component_factory()),
        Route::new("/docs", DocsPage::component_factory()),
        Route::new("/docs/:slug", DocDetailPage::component_factory()),
        Route::new("/benchmarks", BenchmarksPage::component_factory()),
        Route::new("/examples", ExamplesPage::component_factory()),
    ]
}

/// Initialize and mount the application
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize router
    let router = Router::new(get_routes());
    Router::set_global(router);

    // Create root injector with all providers
    let injector = Injector::root();
    injector.provide(vec![
        Provider::singleton::<services::DocsService>(),
        Provider::singleton::<services::SearchService>(),
    ]);

    // Mount the application
    ferric_core::bootstrap::<AppComponent>("app-root")?;

    Ok(())
}
