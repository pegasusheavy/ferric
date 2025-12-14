use ferric_core::prelude::*;
use crate::components::*;
use ferric_markdown::MarkdownService;

#[component(selector = "doc-detail-page")]
pub struct DocDetailPage {
    slug: Signal<String>,
    markdown_service: Inject<MarkdownService>,
    html_content: Signal<String>,
    loading: Signal<bool>,
}

impl Component for DocDetailPage {
    fn new() -> Self {
        Self {
            slug: signal(String::new()),
            markdown_service: Inject::new(Injector::current()),
            html_content: signal(String::new()),
            loading: signal(true),
        }
    }

    fn on_init(&mut self) {
        // Get slug from route params
        let router = Router::current();
        if let Some(params) = router.get_params() {
            if let Some(slug) = params.get("slug") {
                self.slug.set(slug.clone());
                self.load_document(slug);
            }
        }
    }

    fn render(&self) -> Html {
        let loading = self.loading.get();
        let html_content = self.html_content.get();

        html! {
            <div>
                <app-header
                    title="Documentation"
                    subtitle=""
                />

                <main class="max-w-4xl mx-auto px-6 py-12">
                    {if loading {
                        html! {
                            <div class="flex justify-center items-center py-16">
                                <div class="spinner-rust"></div>
                            </div>
                        }
                    } else {
                        html! {
                            <article class="doc-content">
                                <div dangerous_inner_html={html_content}></div>
                            </article>
                        }
                    }}

                    <div class="mt-16 text-center">
                        <a href="/docs" class="btn-rust-outline">
                            "← Back to Documentation"
                        </a>
                    </div>
                </main>

                <app-footer />
            </div>
        }
    }
}

impl DocDetailPage {
    fn load_document(&self, slug: &str) {
        let markdown_service = self.markdown_service.clone();
        let html_content = self.html_content.clone();
        let loading = self.loading.clone();

        // Fetch markdown from docs directory
        let url = format!("/docs/{}.md", slug);

        spawn_local(async move {
            match fetch_markdown(&url).await {
                Ok(markdown) => {
                    let html = markdown_service.get().render(&markdown);
                    html_content.set(html);
                    loading.set(false);
                }
                Err(_) => {
                    html_content.set("<p>Document not found.</p>".to_string());
                    loading.set(false);
                }
            }
        });
    }
}

async fn fetch_markdown(url: &str) -> Result<String, JsValue> {
    use wasm_bindgen::JsCast;
    use web_sys::{Request, RequestInit, Response};

    let mut opts = RequestInit::new();
    opts.method("GET");

    let request = Request::new_with_str_and_init(url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;

    let text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap_or_default())
}

