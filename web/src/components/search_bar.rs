use ferric_core::prelude::*;

#[component(selector = "search-bar")]
pub struct SearchBarComponent {
    search_term: Signal<String>,

    #[output]
    on_search: EventEmitter<String>,
}

impl Component for SearchBarComponent {
    fn new() -> Self {
        Self {
            search_term: signal(String::new()),
            on_search: EventEmitter::new(),
        }
    }

    fn render(&self) -> Html {
        let search_term = self.search_term.clone();
        let on_search = self.on_search.clone();

        html! {
            <div class="search-box max-w-2xl mx-auto">
                <svg class="search-icon w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
                </svg>
                <input
                    type="text"
                    class="search-input"
                    placeholder="Search documentation..."
                    value={search_term.get()}
                    oninput={move |e: InputEvent| {
                        let value = e.target_value();
                        search_term.set(value.clone());
                        on_search.emit(value);
                    }}
                />
            </div>
        }
    }
}

