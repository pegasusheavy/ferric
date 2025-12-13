//! 404 Not Found page component

/// Render the 404 page
/// 
/// The HTML template is located in `not-found.component.html`
/// The styles are located in `not-found.component.scss`
pub fn render() -> String {
    include_str!("not-found.component.html").to_string()
}

