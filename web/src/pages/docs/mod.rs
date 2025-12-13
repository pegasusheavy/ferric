//! Documentation page component

/// Render the documentation main page
///
/// The HTML template is located in `docs.component.html`
/// The styles are located in `docs.component.scss`
pub fn render() -> String {
    include_str!("docs.component.html").to_string()
}

/// Render a specific documentation subpage
pub fn render_page(page: &str) -> String {
    format!(
        r#"<article class="docs">
            <header class="docs__header">
                <h1 class="docs__title">{}</h1>
                <p class="docs__description">Documentation for {}</p>
            </header>
            <section class="docs__section">
                <p class="docs__text">This documentation page is coming soon.</p>
            </section>
        </article>"#,
        page.replace('-', " ").to_uppercase(),
        page
    )
}


/// Render the documentation main page
///
/// The HTML template is located in `docs.component.html`
/// The styles are located in `docs.component.scss`
pub fn render() -> String {
    include_str!("docs.component.html").to_string()
}

/// Render a specific documentation subpage
pub fn render_page(page: &str) -> String {
    format!(
        r#"<article class="docs">
            <header class="docs__header">
                <h1 class="docs__title">{}</h1>
                <p class="docs__description">Documentation for {}</p>
            </header>
            <section class="docs__section">
                <p class="docs__text">This documentation page is coming soon.</p>
            </section>
        </article>"#,
        page.replace('-', " ").to_uppercase(),
        page
    )
}

