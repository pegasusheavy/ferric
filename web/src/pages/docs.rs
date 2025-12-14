use ferric_core::prelude::*;
use crate::components::*;
use crate::services::DocsService;

#[component(selector = "docs-page")]
pub struct DocsPage {
    docs_service: Inject<DocsService>,
    filtered_docs: Signal<Vec<DocMeta>>,
    search_term: Signal<String>,
    active_category: Signal<String>,
}

impl Component for DocsPage {
    fn new() -> Self {
        let docs_service = Inject::new(Injector::current());
        let all_docs = docs_service.get().get_all_docs();

        Self {
            docs_service,
            filtered_docs: signal(all_docs),
            search_term: signal(String::new()),
            active_category: signal("all".to_string()),
        }
    }

    fn on_init(&mut self) {
        self.filter_docs();
    }

    fn render(&self) -> Html {
        let search_term = self.search_term.clone();
        let active_category = self.active_category.clone();
        let filtered_docs = self.filtered_docs.get();

        html! {
            <div>
                <app-header
                    title="Documentation"
                    subtitle="Comprehensive guides and API reference"
                />

                <div class="max-w-6xl mx-auto px-6 -mt-8 relative z-20">
                    <search-bar
                        onsearch={move |term: String| {
                            search_term.set(term);
                            self.filter_docs();
                        }}
                    />
                </div>

                <main class="max-w-7xl mx-auto px-6 py-12">
                    <nav-pills
                        oncategorychange={move |category: String| {
                            active_category.set(category);
                            self.filter_docs();
                        }}
                    />

                    <div class="feature-grid">
                        {for filtered_docs.iter().map(|doc| {
                            html! {
                                <doc-card doc={doc.clone()} />
                            }
                        })}
                    </div>

                    {if filtered_docs.is_empty() {
                        html! {
                            <div class="text-center py-16">
                                <p class="text-xl text-gear">
                                    "No documentation found matching your criteria."
                                </p>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </main>

                <app-footer />
            </div>
        }
    }
}

impl DocsPage {
    fn filter_docs(&self) {
        let docs = self.docs_service.get().get_all_docs();
        let search = self.search_term.get().to_lowercase();
        let category = self.active_category.get();

        let filtered: Vec<DocMeta> = docs
            .into_iter()
            .filter(|doc| {
                // Category filter
                let matches_category = category == "all" || doc.category == category;

                // Search filter
                let matches_search = search.is_empty()
                    || doc.title.to_lowercase().contains(&search)
                    || doc.description.to_lowercase().contains(&search);

                matches_category && matches_search
            })
            .collect();

        self.filtered_docs.set(filtered);
    }
}

