//! Basic Armature SSR example.

use armature_core::prelude::*;
use armature_ssr::prelude::*;
use ferric_core::component::{ComponentMetadata, register_component};
use std::sync::Arc;

// Register components
fn register_components() {
    register_component("app-home", ComponentMetadata {
        selector: "app-home".to_string(),
        template: Some(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Armature SSR Example</title>
                <style>
                    body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
                    h1 { color: #333; }
                    .counter { margin: 20px 0; padding: 20px; background: #f0f0f0; border-radius: 8px; }
                    button { padding: 10px 20px; font-size: 16px; cursor: pointer; }
                </style>
            </head>
            <body>
                <h1>Welcome to Armature SSR</h1>
                <p>This page was rendered on the server!</p>

                <div class="counter">
                    <h2>Interactive Counter</h2>
                    <p>Count: <span id="count">0</span></p>
                    <button onclick="increment()">Increment</button>
                </div>

                <script>
                    let count = 0;
                    function increment() {
                        count++;
                        document.getElementById('count').textContent = count;
                    }
                </script>
            </body>
            </html>
        "#.to_string()),
        styles: vec![],
        ..Default::default()
    });

    register_component("app-about", ComponentMetadata {
        selector: "app-about".to_string(),
        template: Some(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>About - Armature SSR</title>
                <style>
                    body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
                </style>
            </head>
            <body>
                <h1>About Armature SSR</h1>
                <p>Server-side rendering made easy with Rust!</p>
                <a href="/">Back to Home</a>
            </body>
            </html>
        "#.to_string()),
        styles: vec![],
        ..Default::default()
    });
}

#[controller("/")]
struct HomeController {
    ssr_service: Arc<SsrService>,
}

impl HomeController {
    #[get("/")]
    async fn index(&self) -> Result<SsrResponse, Error> {
        self.ssr_service
            .render_component("app-home")
            .with_hydration(HydrationStrategy::Full)
            .await
            .map_err(Into::into)
    }

    #[get("/about")]
    async fn about(&self) -> Result<SsrResponse, Error> {
        self.ssr_service
            .render_component("app-about")
            .await
            .map_err(Into::into)
    }
}

#[module]
struct AppModule;

impl Module for AppModule {
    fn imports(&self) -> Vec<Box<dyn Module>> {
        vec![Box::new(SsrModule::new())]
    }

    fn controllers(&self) -> Vec<Box<dyn Controller>> {
        vec![Box::new(HomeController {
            ssr_service: Arc::new(SsrService::default_config()),
        })]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register components
    register_components();

    // Create application
    let app = Application::new()
        .module(AppModule)
        .build()?;

    println!("🚀 Server starting on http://localhost:3000");
    println!("📍 Routes:");
    println!("   GET  /       - Home page (with hydration)");
    println!("   GET  /about  - About page (static)");

    // Start server
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}

