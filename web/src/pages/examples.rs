use ferric_core::prelude::*;
use crate::components::*;

#[component(selector = "examples-page")]
pub struct ExamplesPage {}

impl Component for ExamplesPage {
    fn new() -> Self {
        Self {}
    }

    fn render(&self) -> Html {
        html! {
            <div>
                <app-header
                    title="Examples"
                    subtitle="Real-world examples and code samples"
                />

                <main class="max-w-7xl mx-auto px-6 py-12">
                    <div class="feature-grid">
                        <doc-card doc={{
                            title: "Todo App",
                            description: "Classic todo list with full CRUD operations",
                            icon: "✅",
                            category: "examples",
                            badges: vec!["Beginner", "Popular"],
                            url: "https://github.com/username/ferric/tree/main/examples/todo-app",
                        }} />
                        <doc-card doc={{
                            title: "Router Example",
                            description: "Multi-page app with routing and navigation",
                            icon: "🛣️",
                            category: "examples",
                            badges: vec!["Intermediate"],
                            url: "https://github.com/username/ferric/tree/main/examples/router-example",
                        }} />
                        <doc-card doc={{
                            title: "Forms Demo",
                            description: "Complex forms with validation",
                            icon: "📝",
                            category: "examples",
                            badges: vec!["Advanced"],
                            url: "https://github.com/username/ferric/tree/main/examples/forms-demo",
                        }} />
                    </div>
                </main>

                <app-footer />
            </div>
        }
    }
}

