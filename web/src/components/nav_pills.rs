use ferric_core::prelude::*;

#[component(selector = "nav-pills")]
pub struct NavPillsComponent {
    active_category: Signal<String>,

    #[output]
    on_category_change: EventEmitter<String>,
}

impl Component for NavPillsComponent {
    fn new() -> Self {
        Self {
            active_category: signal("all".to_string()),
            on_category_change: EventEmitter::new(),
        }
    }

    fn render(&self) -> Html {
        let categories = vec![
            ("all", "All Docs"),
            ("guides", "Guides"),
            ("api", "API Reference"),
            ("tutorials", "Tutorials"),
            ("examples", "Examples"),
        ];

        html! {
            <nav class="flex gap-2 flex-wrap mb-8 justify-center">
                {for categories.iter().map(|(id, label)| {
                    let active = self.active_category.get() == *id;
                    let class_name = if active {
                        "nav-item-rust nav-item-active"
                    } else {
                        "nav-item-rust"
                    };

                    let active_category = self.active_category.clone();
                    let on_category_change = self.on_category_change.clone();
                    let id_str = id.to_string();

                    html! {
                        <button
                            class={class_name}
                            data-category={id}
                            onclick={move |_| {
                                active_category.set(id_str.clone());
                                on_category_change.emit(id_str.clone());
                            }}
                        >
                            {label}
                        </button>
                    }
                })}
            </nav>
        }
    }
}

