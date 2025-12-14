//! SEO metadata management
//!
//! Handles meta tags, Open Graph, Twitter Cards, and structured data

use serde::{Deserialize, Serialize};

/// SEO metadata for a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoMetadata {
    /// Page title
    pub title: String,
    
    /// Meta description
    pub description: String,
    
    /// Keywords
    pub keywords: Vec<String>,
    
    /// Canonical URL
    pub canonical_url: String,
    
    /// Open Graph metadata
    pub og: OpenGraphMetadata,
    
    /// Twitter Card metadata
    pub twitter: TwitterCardMetadata,
    
    /// Structured data (JSON-LD)
    pub structured_data: Vec<StructuredData>,
    
    /// Robots directives
    pub robots: RobotsDirectives,
    
    /// Language code
    pub language: String,
    
    /// AI-specific metadata
    pub ai: AiMetadata,
}

/// Open Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenGraphMetadata {
    pub title: String,
    pub description: String,
    pub url: String,
    pub image: String,
    pub image_alt: String,
    pub site_name: String,
    pub og_type: String, // "website", "article", etc.
    pub locale: String,
}

/// Twitter Card metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterCardMetadata {
    pub card: String, // "summary", "summary_large_image"
    pub site: String,
    pub creator: String,
    pub title: String,
    pub description: String,
    pub image: String,
    pub image_alt: String,
}

/// Robots directives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsDirectives {
    pub index: bool,
    pub follow: bool,
    pub archive: bool,
    pub snippet: bool,
    pub max_snippet: Option<usize>,
    pub max_image_preview: Option<String>, // "none", "standard", "large"
}

/// AI-specific metadata for LLM crawlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMetadata {
    /// Content for AI training/indexing
    pub ai_content_summary: String,
    
    /// Key topics for AI understanding
    pub ai_topics: Vec<String>,
    
    /// Content type for AI
    pub ai_content_type: String, // "documentation", "tutorial", "api-reference"
    
    /// Difficulty level
    pub ai_difficulty: String, // "beginner", "intermediate", "advanced"
    
    /// Expected reading time (minutes)
    pub ai_reading_time: usize,
    
    /// Related topics
    pub ai_related_topics: Vec<String>,
    
    /// Prerequisites
    pub ai_prerequisites: Vec<String>,
}

/// Structured data types (Schema.org)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum StructuredData {
    WebSite {
        name: String,
        url: String,
        description: String,
        potential_action: Option<SearchAction>,
    },
    TechArticle {
        headline: String,
        description: String,
        author: Author,
        date_published: String,
        date_modified: String,
        keywords: Vec<String>,
        proficiency_level: String,
    },
    SoftwareApplication {
        name: String,
        description: String,
        application_category: String,
        operating_system: String,
        programming_language: String,
    },
    BreadcrumbList {
        item_list_element: Vec<BreadcrumbItem>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAction {
    #[serde(rename = "@type")]
    pub action_type: String,
    pub target: String,
    #[serde(rename = "query-input")]
    pub query_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    #[serde(rename = "@type")]
    pub author_type: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    #[serde(rename = "@type")]
    pub item_type: String,
    pub position: usize,
    pub item: BreadcrumbItemDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItemDetails {
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
}

impl SeoMetadata {
    /// Create SEO metadata for the homepage
    pub fn homepage() -> Self {
        Self {
            title: "Ferric - Angular-inspired Rust Web Framework".to_string(),
            description: "A modern, type-safe web framework for Rust with WASM support. Build reactive web applications with Angular-like patterns and full type safety.".to_string(),
            keywords: vec![
                "Rust".to_string(),
                "WASM".to_string(),
                "WebAssembly".to_string(),
                "Web Framework".to_string(),
                "Frontend Framework".to_string(),
                "Reactive Programming".to_string(),
                "Type Safety".to_string(),
                "Angular".to_string(),
                "Ferric".to_string(),
            ],
            canonical_url: "https://pegasusheavy.github.io/ferric/".to_string(),
            og: OpenGraphMetadata {
                title: "Ferric Framework".to_string(),
                description: "Angular-inspired Rust web framework with WASM support".to_string(),
                url: "https://pegasusheavy.github.io/ferric/".to_string(),
                image: "https://pegasusheavy.github.io/ferric/assets/og-image.png".to_string(),
                image_alt: "Ferric Framework - Rust Web Development".to_string(),
                site_name: "Ferric Framework".to_string(),
                og_type: "website".to_string(),
                locale: "en_US".to_string(),
            },
            twitter: TwitterCardMetadata {
                card: "summary_large_image".to_string(),
                site: "@ferric_framework".to_string(),
                creator: "@pegasusheavy".to_string(),
                title: "Ferric - Rust Web Framework".to_string(),
                description: "Build reactive web apps with Rust and WASM".to_string(),
                image: "https://pegasusheavy.github.io/ferric/assets/twitter-card.png".to_string(),
                image_alt: "Ferric Framework".to_string(),
            },
            structured_data: vec![
                StructuredData::WebSite {
                    name: "Ferric Framework".to_string(),
                    url: "https://pegasusheavy.github.io/ferric/".to_string(),
                    description: "Angular-inspired Rust web framework".to_string(),
                    potential_action: Some(SearchAction {
                        action_type: "SearchAction".to_string(),
                        target: "https://pegasusheavy.github.io/ferric/search?q={search_term_string}".to_string(),
                        query_input: "required name=search_term_string".to_string(),
                    }),
                },
                StructuredData::SoftwareApplication {
                    name: "Ferric".to_string(),
                    description: "Modern web framework for Rust with WASM support".to_string(),
                    application_category: "DeveloperApplication".to_string(),
                    operating_system: "Cross-platform".to_string(),
                    programming_language: "Rust".to_string(),
                },
            ],
            robots: RobotsDirectives {
                index: true,
                follow: true,
                archive: true,
                snippet: true,
                max_snippet: Some(160),
                max_image_preview: Some("large".to_string()),
            },
            language: "en".to_string(),
            ai: AiMetadata {
                ai_content_summary: "Ferric is a modern web framework for Rust inspired by Angular. It provides reactive programming, type safety, WASM support, and familiar patterns for building web applications.".to_string(),
                ai_topics: vec![
                    "Rust programming".to_string(),
                    "WebAssembly".to_string(),
                    "Web development".to_string(),
                    "Frontend frameworks".to_string(),
                    "Reactive programming".to_string(),
                ],
                ai_content_type: "documentation".to_string(),
                ai_difficulty: "intermediate".to_string(),
                ai_reading_time: 5,
                ai_related_topics: vec![
                    "TypeScript".to_string(),
                    "Angular".to_string(),
                    "React".to_string(),
                    "Vue.js".to_string(),
                ],
                ai_prerequisites: vec![
                    "Basic Rust knowledge".to_string(),
                    "Web development fundamentals".to_string(),
                ],
            },
        }
    }

    /// Create SEO metadata for a documentation page
    pub fn documentation(title: &str, description: &str, path: &str, keywords: Vec<String>) -> Self {
        let full_title = format!("{} | Ferric Framework", title);
        let url = format!("https://pegasusheavy.github.io/ferric/{}", path);
        
        Self {
            title: full_title.clone(),
            description: description.to_string(),
            keywords,
            canonical_url: url.clone(),
            og: OpenGraphMetadata {
                title: full_title.clone(),
                description: description.to_string(),
                url: url.clone(),
                image: "https://pegasusheavy.github.io/ferric/assets/og-docs.png".to_string(),
                image_alt: format!("{} - Ferric Documentation", title),
                site_name: "Ferric Framework".to_string(),
                og_type: "article".to_string(),
                locale: "en_US".to_string(),
            },
            twitter: TwitterCardMetadata {
                card: "summary".to_string(),
                site: "@ferric_framework".to_string(),
                creator: "@pegasusheavy".to_string(),
                title: full_title,
                description: description.to_string(),
                image: "https://pegasusheavy.github.io/ferric/assets/twitter-docs.png".to_string(),
                image_alt: format!("{} Documentation", title),
            },
            structured_data: vec![
                StructuredData::TechArticle {
                    headline: title.to_string(),
                    description: description.to_string(),
                    author: Author {
                        author_type: "Organization".to_string(),
                        name: "Ferric Contributors".to_string(),
                        url: Some("https://github.com/pegasusheavy/ferric".to_string()),
                    },
                    date_published: "2024-01-01T00:00:00Z".to_string(),
                    date_modified: "2024-12-15T00:00:00Z".to_string(),
                    keywords: vec![],
                    proficiency_level: "Intermediate".to_string(),
                },
            ],
            robots: RobotsDirectives {
                index: true,
                follow: true,
                archive: true,
                snippet: true,
                max_snippet: Some(200),
                max_image_preview: Some("standard".to_string()),
            },
            language: "en".to_string(),
            ai: AiMetadata {
                ai_content_summary: description.to_string(),
                ai_topics: vec![
                    "Rust".to_string(),
                    "Web Framework".to_string(),
                    "Documentation".to_string(),
                ],
                ai_content_type: "documentation".to_string(),
                ai_difficulty: "intermediate".to_string(),
                ai_reading_time: 10,
                ai_related_topics: vec![],
                ai_prerequisites: vec!["Rust basics".to_string()],
            },
        }
    }

    /// Generate HTML meta tags
    pub fn to_html_meta_tags(&self) -> String {
        let mut tags = String::new();
        
        // Basic meta tags
        tags.push_str(&format!(r#"<title>{}</title>"#, html_escape(&self.title)));
        tags.push_str(&format!(r#"<meta name="description" content="{}">"#, html_escape(&self.description)));
        tags.push_str(&format!(r#"<meta name="keywords" content="{}">"#, self.keywords.join(", ")));
        tags.push_str(&format!(r#"<link rel="canonical" href="{}">"#, self.canonical_url));
        tags.push_str(&format!(r#"<html lang="{}">"#, self.language));
        
        // Robots
        let robots = format!(
            "{}, {}",
            if self.robots.index { "index" } else { "noindex" },
            if self.robots.follow { "follow" } else { "nofollow" }
        );
        tags.push_str(&format!(r#"<meta name="robots" content="{}">"#, robots));
        
        if let Some(max) = self.robots.max_snippet {
            tags.push_str(&format!(r#"<meta name="robots" content="max-snippet:{}">"#, max));
        }
        
        if let Some(ref preview) = self.robots.max_image_preview {
            tags.push_str(&format!(r#"<meta name="robots" content="max-image-preview:{}">"#, preview));
        }
        
        // Open Graph
        tags.push_str(&format!(r#"<meta property="og:title" content="{}">"#, html_escape(&self.og.title)));
        tags.push_str(&format!(r#"<meta property="og:description" content="{}">"#, html_escape(&self.og.description)));
        tags.push_str(&format!(r#"<meta property="og:url" content="{}">"#, self.og.url));
        tags.push_str(&format!(r#"<meta property="og:image" content="{}">"#, self.og.image));
        tags.push_str(&format!(r#"<meta property="og:image:alt" content="{}">"#, html_escape(&self.og.image_alt)));
        tags.push_str(&format!(r#"<meta property="og:site_name" content="{}">"#, html_escape(&self.og.site_name)));
        tags.push_str(&format!(r#"<meta property="og:type" content="{}">"#, self.og.og_type));
        tags.push_str(&format!(r#"<meta property="og:locale" content="{}">"#, self.og.locale));
        
        // Twitter Card
        tags.push_str(&format!(r#"<meta name="twitter:card" content="{}">"#, self.twitter.card));
        tags.push_str(&format!(r#"<meta name="twitter:site" content="{}">"#, self.twitter.site));
        tags.push_str(&format!(r#"<meta name="twitter:creator" content="{}">"#, self.twitter.creator));
        tags.push_str(&format!(r#"<meta name="twitter:title" content="{}">"#, html_escape(&self.twitter.title)));
        tags.push_str(&format!(r#"<meta name="twitter:description" content="{}">"#, html_escape(&self.twitter.description)));
        tags.push_str(&format!(r#"<meta name="twitter:image" content="{}">"#, self.twitter.image));
        tags.push_str(&format!(r#"<meta name="twitter:image:alt" content="{}">"#, html_escape(&self.twitter.image_alt)));
        
        // AI-specific meta tags
        tags.push_str(&format!(r#"<meta name="ai:content_summary" content="{}">"#, html_escape(&self.ai.ai_content_summary)));
        tags.push_str(&format!(r#"<meta name="ai:topics" content="{}">"#, self.ai.ai_topics.join(", ")));
        tags.push_str(&format!(r#"<meta name="ai:content_type" content="{}">"#, self.ai.ai_content_type));
        tags.push_str(&format!(r#"<meta name="ai:difficulty" content="{}">"#, self.ai.ai_difficulty));
        tags.push_str(&format!(r#"<meta name="ai:reading_time" content="{}">"#, self.ai.ai_reading_time));
        
        if !self.ai.ai_related_topics.is_empty() {
            tags.push_str(&format!(r#"<meta name="ai:related_topics" content="{}">"#, self.ai.ai_related_topics.join(", ")));
        }
        
        if !self.ai.ai_prerequisites.is_empty() {
            tags.push_str(&format!(r#"<meta name="ai:prerequisites" content="{}">"#, self.ai.ai_prerequisites.join(", ")));
        }
        
        tags
    }

    /// Generate JSON-LD structured data
    pub fn to_json_ld(&self) -> String {
        let json = serde_json::to_string_pretty(&self.structured_data).unwrap_or_default();
        format!(r#"<script type="application/ld+json">{}</script>"#, json)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homepage_seo() {
        let seo = SeoMetadata::homepage();
        assert!(seo.title.contains("Ferric"));
        assert!(seo.robots.index);
        assert!(seo.robots.follow);
    }

    #[test]
    fn test_html_escape() {
        let escaped = html_escape("<script>alert('xss')</script>");
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
    }
}

