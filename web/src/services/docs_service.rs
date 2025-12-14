use ferric_core::di::Injectable;
use crate::components::DocMeta;

#[injectable]
pub struct DocsService {}

impl Injectable for DocsService {
    fn create(_injector: &ferric_core::di::Injector) -> Self {
        Self {}
    }
}

impl DocsService {
    pub fn get_all_docs(&self) -> Vec<DocMeta> {
        vec![
            DocMeta {
                title: "Getting Started".to_string(),
                description: "Quick start guide to building your first Ferric application".to_string(),
                icon: "🚀".to_string(),
                category: "guides".to_string(),
                badges: vec!["Beginner".to_string(), "Essential".to_string()],
                url: "/docs/getting-started".to_string(),
            },
            DocMeta {
                title: "Core Concepts".to_string(),
                description: "Understand the fundamental concepts behind Ferric".to_string(),
                icon: "🧠".to_string(),
                category: "guides".to_string(),
                badges: vec!["Intermediate".to_string(), "Essential".to_string()],
                url: "/docs/core-concepts".to_string(),
            },
            DocMeta {
                title: "Components".to_string(),
                description: "Learn how to build reusable UI components".to_string(),
                icon: "🧩".to_string(),
                category: "guides".to_string(),
                badges: vec!["Beginner".to_string()],
                url: "/docs/components".to_string(),
            },
            DocMeta {
                title: "Reactivity System".to_string(),
                description: "Signals, computed values, and effects for reactive programming".to_string(),
                icon: "⚡".to_string(),
                category: "guides".to_string(),
                badges: vec!["Intermediate".to_string(), "Popular".to_string()],
                url: "/docs/reactivity".to_string(),
            },
            DocMeta {
                title: "Routing".to_string(),
                description: "Client-side routing with guards and resolvers".to_string(),
                icon: "🛣️".to_string(),
                category: "guides".to_string(),
                badges: vec!["Intermediate".to_string()],
                url: "/docs/routing".to_string(),
            },
            DocMeta {
                title: "Forms & Validation".to_string(),
                description: "Type-safe forms with automatic validation".to_string(),
                icon: "📝".to_string(),
                category: "guides".to_string(),
                badges: vec!["Advanced".to_string()],
                url: "/docs/forms".to_string(),
            },
            DocMeta {
                title: "HTTP Client".to_string(),
                description: "Make HTTP requests with interceptors and retry logic".to_string(),
                icon: "🌐".to_string(),
                category: "api".to_string(),
                badges: vec!["Intermediate".to_string()],
                url: "/docs/http".to_string(),
            },
            DocMeta {
                title: "Dependency Injection".to_string(),
                description: "Hierarchical DI system for managing services".to_string(),
                icon: "💉".to_string(),
                category: "guides".to_string(),
                badges: vec!["Advanced".to_string(), "Essential".to_string()],
                url: "/docs/di-architecture".to_string(),
            },
            DocMeta {
                title: "Server-Side Rendering".to_string(),
                description: "SSR with hydration for all major Rust frameworks".to_string(),
                icon: "🖥️".to_string(),
                category: "guides".to_string(),
                badges: vec!["Advanced".to_string(), "Popular".to_string()],
                url: "/docs/ssr-integrations-guide".to_string(),
            },
            DocMeta {
                title: "Async Programming".to_string(),
                description: "Futures, promises, and async utilities".to_string(),
                icon: "🔄".to_string(),
                category: "guides".to_string(),
                badges: vec!["Advanced".to_string()],
                url: "/docs/async-support-guide".to_string(),
            },
            DocMeta {
                title: "Macros Reference".to_string(),
                description: "Complete guide to Ferric's powerful macros".to_string(),
                icon: "🎯".to_string(),
                category: "api".to_string(),
                badges: vec!["Intermediate".to_string()],
                url: "/docs/macros-summary".to_string(),
            },
            DocMeta {
                title: "Testing Guide".to_string(),
                description: "Unit testing, integration tests, and doctests".to_string(),
                icon: "🧪".to_string(),
                category: "guides".to_string(),
                badges: vec!["Intermediate".to_string()],
                url: "/docs/doctest-guide".to_string(),
            },
        ]
    }
}

