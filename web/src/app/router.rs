//! Client-side routing

use web_sys::Window;

/// Application routes
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Home,
    Docs,
    DocsPage(String),
    Guide,
    Api,
    Examples,
    NotFound,
}

/// Router for handling client-side navigation
pub struct Router;

impl Router {
    /// Parse route from window location
    pub fn parse_route(window: &Window) -> Route {
        if let Ok(location) = window.location().pathname() {
            Self::from_path(&location)
        } else {
            Route::Home
        }
    }

    /// Parse route from path string
    pub fn from_path(path: &str) -> Route {
        let path = path.trim_start_matches('/');

        match path {
            "" | "home" => Route::Home,
            "docs" => Route::Docs,
            "guide" => Route::Guide,
            "api" => Route::Api,
            "examples" => Route::Examples,
            _ if path.starts_with("docs/") => {
                let page = path.strip_prefix("docs/").unwrap_or("");
                Route::DocsPage(page.to_string())
            }
            _ => Route::NotFound,
        }
    }
}
