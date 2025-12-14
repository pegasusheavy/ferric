use ferric_core::prelude::*;
use crate::components::*;

#[component(selector = "benchmarks-page")]
pub struct BenchmarksPage {}

impl Component for BenchmarksPage {
    fn new() -> Self {
        Self {}
    }

    fn render(&self) -> Html {
        html! {
            <div>
                <app-header
                    title="Benchmarks"
                    subtitle="Performance comparison with other frameworks"
                />

                <main class="max-w-7xl mx-auto px-6 py-12">
                    <div class="alert-info mb-8">
                        <p class="font-semibold">"Performance benchmarks coming soon!"</p>
                        <p>"We're currently running comprehensive benchmarks against React, Vue, and other frameworks."</p>
                    </div>

                    <section class="mb-16">
                        <h2 class="text-3xl font-bold mb-6 text-gradient-rust">"Why Ferric is Fast"</h2>
                        <div class="grid md:grid-cols-2 gap-6">
                            <feature-card
                                icon="🦀"
                                title="Rust Performance"
                                description="Compiled to WebAssembly for near-native performance in the browser."
                            />
                            <feature-card
                                icon="⚡"
                                title="Fine-Grained Reactivity"
                                description="Precise updates without virtual DOM diffing overhead."
                            />
                            <feature-card
                                icon="📦"
                                title="Small Bundle Size"
                                description="Aggressive tree-shaking and compression for minimal payload."
                            />
                            <feature-card
                                icon="🔧"
                                title="Zero-Cost Abstractions"
                                description="High-level APIs with no runtime overhead."
                            />
                        </div>
                    </section>
                </main>

                <app-footer />
            </div>
        }
    }
}

