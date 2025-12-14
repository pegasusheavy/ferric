use ferric_core::prelude::*;
use crate::components::*;

#[component(selector = "home-page")]
pub struct HomePage {}

impl Component for HomePage {
    fn new() -> Self {
        Self {}
    }

    fn render(&self) -> Html {
        html! {
            <div>
                <app-header
                    title="Ferric Framework"
                    subtitle="Build blazingly fast web applications with Rust"
                />

                <main class="max-w-7xl mx-auto px-6 py-12">
                    <section class="text-center mb-16">
                        <h2 class="text-4xl font-bold mb-6 text-gradient-rust">
                            "A Modern Web Framework for Rust"
                        </h2>
                        <p class="text-xl text-gear max-w-3xl mx-auto mb-8">
                            "Ferric brings the power of Rust to web development with an Angular-inspired architecture, reactive programming, and WebAssembly performance."
                        </p>
                        <div class="flex gap-4 justify-center">
                            <a href="/docs" class="btn-rust">
                                "Get Started"
                            </a>
                            <a href="/examples" class="btn-rust-outline">
                                "View Examples"
                            </a>
                        </div>
                    </section>

                    <section class="mb-16">
                        <h2 class="text-3xl font-bold text-center mb-8 text-gradient-rust">
                            "Why Choose Ferric?"
                        </h2>
                        <div class="grid md:grid-cols-3 gap-6">
                            <feature-card
                                icon="⚡"
                                title="Blazingly Fast"
                                description="Compiled to WebAssembly for native performance. 10x faster than JavaScript frameworks."
                            />
                            <feature-card
                                icon="🔒"
                                title="Type-Safe"
                                description="Rust's type system prevents entire classes of bugs at compile-time."
                            />
                            <feature-card
                                icon="🧩"
                                title="Modular"
                                description="Pick only the features you need. Tree-shaking for minimal bundle sizes."
                            />
                        </div>
                    </section>

                    <section class="mb-16">
                        <h2 class="text-3xl font-bold text-center mb-8">
                            "Quick Example"
                        </h2>
                        <div class="max-w-3xl mx-auto">
                            <div class="code-rust">
                                <div class="code-header">
                                    <span>"counter.rs"</span>
                                </div>
                                <pre class="p-4 overflow-x-auto"><code class="text-sm">{r#"use ferric_core::prelude::*;

#[component(selector = "counter")]
struct Counter {
    count: Signal<i32>,
}

impl Component for Counter {
    fn new() -> Self {
        Self {
            count: signal(0),
        }
    }

    fn render(&self) -> Html {
        let count = self.count.clone();

        html! {
            <div class="p-4">
                <p>"Count: " {count.get()}</p>
                <button
                    class="btn-rust"
                    onclick={move |_| count.update(|n| n + 1)}
                >
                    "Increment"
                </button>
            </div>
        }
    }
}"#}</code></pre>
                            </div>
                        </div>
                    </section>

                    <section>
                        <h2 class="text-3xl font-bold text-center mb-8 text-gradient-rust">
                            "Featured Documentation"
                        </h2>
                        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
                            <doc-card doc={{
                                title: "Getting Started",
                                description: "Quick start guide to building your first Ferric application",
                                icon: "🚀",
                                category: "guides",
                                badges: vec!["Beginner", "Essential"],
                                url: "/docs/getting-started",
                            }} />
                            <doc-card doc={{
                                title: "Reactivity System",
                                description: "Signals, computed values, and effects for reactive programming",
                                icon: "⚡",
                                category: "guides",
                                badges: vec!["Intermediate", "Popular"],
                                url: "/docs/reactivity",
                            }} />
                            <doc-card doc={{
                                title: "Components",
                                description: "Learn how to build reusable UI components",
                                icon: "🧩",
                                category: "guides",
                                badges: vec!["Beginner"],
                                url: "/docs/components",
                            }} />
                        </div>
                        <div class="text-center mt-8">
                            <a href="/docs" class="btn-rust">
                                "View All Documentation"
                            </a>
                        </div>
                    </section>
                </main>

                <app-footer />
            </div>
        }
    }
}

