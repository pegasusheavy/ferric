//! Sitemap generation for SEO

use serde::{Deserialize, Serialize};

/// Sitemap URL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapUrl {
    pub loc: String,
    pub lastmod: String,
    pub changefreq: String,
    pub priority: f32,
}

/// Generate sitemap.xml
pub fn generate_sitemap() -> String {
    let urls = vec![
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "weekly".to_string(),
            priority: 1.0,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/getting-started".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.9,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/components".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.8,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/routing".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.8,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/forms".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.8,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/http".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.8,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/dependency-injection".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.8,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/ssr".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.7,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/testing".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.7,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/docs/deployment".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "monthly".to_string(),
            priority: 0.7,
        },
        SitemapUrl {
            loc: "https://pegasusheavy.github.io/ferric/benchmarks".to_string(),
            lastmod: "2024-12-15".to_string(),
            changefreq: "weekly".to_string(),
            priority: 0.6,
        },
    ];

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

    for url in urls {
        xml.push_str("<url>");
        xml.push_str(&format!("<loc>{}</loc>", url.loc));
        xml.push_str(&format!("<lastmod>{}</lastmod>", url.lastmod));
        xml.push_str(&format!("<changefreq>{}</changefreq>", url.changefreq));
        xml.push_str(&format!("<priority>{}</priority>", url.priority));
        xml.push_str("</url>");
    }

    xml.push_str("</urlset>");
    xml
}

