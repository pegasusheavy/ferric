use ferric_core::di::Injectable;

#[injectable]
pub struct SearchService {}

impl Injectable for SearchService {
    fn create(_injector: &ferric_core::di::Injector) -> Self {
        Self {}
    }
}

impl SearchService {
    pub fn search(&self, query: &str, items: &[String]) -> Vec<String> {
        let query_lower = query.to_lowercase();
        items
            .iter()
            .filter(|item| item.to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }
}

