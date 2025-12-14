use ferric_core::prelude::*;

#[component(selector = "app-footer")]
pub struct FooterComponent {}

impl Component for FooterComponent {
    fn new() -> Self {
        Self {}
    }

    fn render(&self) -> Html {
        html! {
            <footer class="footer-rust mt-16">
                <div class="max-w-6xl mx-auto">
                    <div class="grid md:grid-cols-4 gap-8 mb-8">
                        <div>
                            <h3 class="text-rust-400 font-bold mb-4">"Documentation"</h3>
                            <ul class="space-y-2">
                                <li><a href="/docs/getting-started" class="footer-link">"Getting Started"</a></li>
                                <li><a href="/docs/core-concepts" class="footer-link">"Core Concepts"</a></li>
                                <li><a href="/docs/api-reference" class="footer-link">"API Reference"</a></li>
                            </ul>
                        </div>
                        <div>
                            <h3 class="text-rust-400 font-bold mb-4">"Community"</h3>
                            <ul class="space-y-2">
                                <li><a href="https://github.com/username/ferric" class="footer-link">"GitHub"</a></li>
                                <li><a href="https://discord.gg/ferric" class="footer-link">"Discord"</a></li>
                                <li><a href="https://twitter.com/ferric_rs" class="footer-link">"Twitter"</a></li>
                            </ul>
                        </div>
                        <div>
                            <h3 class="text-rust-400 font-bold mb-4">"Resources"</h3>
                            <ul class="space-y-2">
                                <li><a href="/benchmarks" class="footer-link">"Benchmarks"</a></li>
                                <li><a href="/examples" class="footer-link">"Examples"</a></li>
                                <li><a href="/blog" class="footer-link">"Blog"</a></li>
                            </ul>
                        </div>
                        <div>
                            <h3 class="text-rust-400 font-bold mb-4">"Legal"</h3>
                            <ul class="space-y-2">
                                <li><a href="/license" class="footer-link">"License (MIT)"</a></li>
                                <li><a href="/privacy" class="footer-link">"Privacy"</a></li>
                                <li><a href="/contributing" class="footer-link">"Contributing"</a></li>
                            </ul>
                        </div>
                    </div>
                    <div class="border-t border-gear pt-8 text-center text-rust-300">
                        <p>"Built with 🦀 Rust and ❤️ by the Ferric community"</p>
                        <p class="mt-2 text-sm">"© 2024 Ferric Framework. All rights reserved."</p>
                    </div>
                </div>
            </footer>
        }
    }
}

