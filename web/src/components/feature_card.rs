use ferric_core::prelude::*;

#[component(selector = "feature-card")]
pub struct FeatureCardComponent {
    #[input]
    icon: String,

    #[input]
    title: String,

    #[input]
    description: String,
}

impl Component for FeatureCardComponent {
    fn new() -> Self {
        Self {
            icon: String::new(),
            title: String::new(),
            description: String::new(),
        }
    }

    fn render(&self) -> Html {
        html! {
            <div class="card-rust">
                <div class="card-header-rust">
                    {&self.icon} " " {&self.title}
                </div>
                <div class="card-body">
                    <p class="text-gear">
                        {&self.description}
                    </p>
                </div>
            </div>
        }
    }
}
