use ferric_core::prelude::*;

#[component(selector = "app-header")]
pub struct HeaderComponent {
    #[input]
    title: String,

    #[input]
    subtitle: String,
}

impl Component for HeaderComponent {
    fn new() -> Self {
        Self {
            title: "Ferric Documentation".to_string(),
            subtitle: "Build blazingly fast web applications with Rust".to_string(),
        }
    }

    fn render(&self) -> Html {
        html! {
            <header class="header-rust">
                <div class="absolute inset-0 bg-gears opacity-10"></div>
                <div class="relative z-10 max-w-6xl mx-auto">
                    <h1 class="text-5xl md:text-6xl font-bold mb-4 animate-ferris-wave inline-block">
                        "🦀 " {&self.title}
                    </h1>
                    <p class="text-xl md:text-2xl opacity-90 max-w-3xl mx-auto">
                        {&self.subtitle}
                    </p>
                    <div class="mt-8 flex gap-4 justify-center flex-wrap">
                        <a href="/" class="btn-rust">
                            "Home"
                        </a>
                        <a href="/docs" class="btn-rust-outline bg-white/10 border-white text-white hover:bg-white hover:text-rust-600">
                            "Documentation"
                        </a>
                        <a href="/benchmarks" class="btn-rust-outline bg-white/10 border-white text-white hover:bg-white hover:text-rust-600">
                            "Benchmarks"
                        </a>
                        <a href="https://github.com/username/ferric" class="btn-rust-outline bg-white/10 border-white text-white hover:bg-white hover:text-rust-600">
                            "GitHub"
                        </a>
                    </div>
                </div>
            </header>
        }
    }
}

