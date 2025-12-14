use ferric_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    pub title: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub badges: Vec<String>,
    pub url: String,
}

#[component(selector = "doc-card")]
pub struct DocCardComponent {
    #[input]
    pub doc: DocMeta,
}

impl Component for DocCardComponent {
    fn new() -> Self {
        Self {
            doc: DocMeta {
                title: String::new(),
                description: String::new(),
                icon: String::new(),
                category: String::new(),
                badges: Vec::new(),
                url: String::new(),
            },
        }
    }

    fn render(&self) -> Html {
        html! {
            <article class="feature-card" data-category={&self.doc.category}>
                <div class="feature-icon">
                    {&self.doc.icon}
                </div>
                <h3 class="text-xl font-bold text-gear-dark mb-2">
                    {&self.doc.title}
                </h3>
                <p class="text-gear mb-4">
                    {&self.doc.description}
                </p>
                <div class="flex gap-2 flex-wrap mb-4">
                    {for self.doc.badges.iter().enumerate().map(|(i, badge)| {
                        let class = if i == 0 { "badge-rust" } else if i == 1 { "badge-ferric" } else { "badge-crab" };
                        html! {
                            <span class={class}>{badge}</span>
                        }
                    })}
                </div>
                <a href={&self.doc.url} class="link-rust">
                    "Read more →"
                </a>
            </article>
        }
    }
}

