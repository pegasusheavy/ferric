//! Hydration strategies for server-rendered components.
//!
//! This module provides support for:
//! - Full hydration: Rehydrate entire server-rendered pages with interactivity
//! - Partial hydration (Islands): Selectively hydrate interactive components
//!
//! ## Full Hydration
//!
//! Full hydration attaches event listeners and restores state to all components
//! in the server-rendered HTML.
//!
//! ```ignore
//! use ferric_ssr::hydration::{HydrationStrategy, FullHydration};
//!
//! let strategy = FullHydration::new()
//!     .with_root_id("app")
//!     .with_state_script_id("__FERRIC_STATE__");
//! ```
//!
//! ## Partial Hydration (Islands)
//!
//! Island architecture allows you to specify which components should be hydrated,
//! leaving static content as-is for better performance.
//!
//! ```ignore
//! use ferric_ssr::hydration::{HydrationStrategy, PartialHydration, Island};
//!
//! let strategy = PartialHydration::new()
//!     .add_island(Island::new("interactive-button")
//!         .with_priority(IslandPriority::High))
//!     .add_island(Island::new("comment-form")
//!         .with_loading(IslandLoading::Lazy));
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Strategy for hydrating server-rendered content.
pub trait HydrationStrategy {
    /// Get the strategy type name.
    fn strategy_type(&self) -> &str;

    /// Generate hydration markers for rendered HTML.
    fn generate_markers(&self, component_id: &str) -> HydrationMarkers;

    /// Generate the client-side hydration script.
    fn generate_hydration_script(&self, config: &HydrationConfig) -> String;

    /// Check if a component should be hydrated.
    fn should_hydrate(&self, component_id: &str) -> bool;
}

/// Markers to embed in HTML for hydration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationMarkers {
    /// Data attributes to add to the component root element.
    pub attributes: HashMap<String, String>,

    /// Whether this component should be hydrated.
    pub hydrate: bool,

    /// Priority for hydration (higher = sooner).
    pub priority: IslandPriority,
}

impl HydrationMarkers {
    /// Create markers for a hydrated component.
    pub fn hydrated(component_id: &str) -> Self {
        let mut attributes = HashMap::new();
        attributes.insert("data-ferric-hydrate".to_string(), "true".to_string());
        attributes.insert("data-ferric-id".to_string(), component_id.to_string());

        Self {
            attributes,
            hydrate: true,
            priority: IslandPriority::Normal,
        }
    }

    /// Create markers for a static (non-hydrated) component.
    pub fn static_component() -> Self {
        Self {
            attributes: HashMap::new(),
            hydrate: false,
            priority: IslandPriority::None,
        }
    }

    /// Set the hydration priority.
    pub fn with_priority(mut self, priority: IslandPriority) -> Self {
        if self.hydrate {
            self.attributes.insert(
                "data-ferric-priority".to_string(),
                priority.as_str().to_string(),
            );
        }
        self.priority = priority;
        self
    }

    /// Convert to HTML attribute string.
    pub fn to_html_attrs(&self) -> String {
        self.attributes
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, html_escape::encode_text(v)))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Configuration for hydration.
#[derive(Debug, Clone)]
pub struct HydrationConfig {
    /// Root element ID where the app is mounted.
    pub root_id: String,

    /// ID of the script tag containing serialized state.
    pub state_script_id: String,

    /// Whether to enable debug logging during hydration.
    pub debug: bool,

    /// Timeout for hydration (milliseconds).
    pub timeout_ms: u32,
}

impl Default for HydrationConfig {
    fn default() -> Self {
        Self {
            root_id: "app".to_string(),
            state_script_id: "__FERRIC_STATE__".to_string(),
            debug: false,
            timeout_ms: 10000,
        }
    }
}

impl HydrationConfig {
    /// Create a new hydration config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the root element ID.
    pub fn with_root_id(mut self, id: impl Into<String>) -> Self {
        self.root_id = id.into();
        self
    }

    /// Set the state script ID.
    pub fn with_state_script_id(mut self, id: impl Into<String>) -> Self {
        self.state_script_id = id.into();
        self
    }

    /// Enable debug logging.
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set hydration timeout.
    pub fn with_timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// Full hydration strategy - hydrates the entire application.
#[derive(Debug, Clone)]
pub struct FullHydration {
    config: HydrationConfig,
}

impl FullHydration {
    /// Create a new full hydration strategy.
    pub fn new() -> Self {
        Self {
            config: HydrationConfig::default(),
        }
    }

    /// Configure the hydration.
    pub fn with_config(mut self, config: HydrationConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the root element ID.
    pub fn with_root_id(mut self, id: impl Into<String>) -> Self {
        self.config.root_id = id.into();
        self
    }

    /// Set the state script ID.
    pub fn with_state_script_id(mut self, id: impl Into<String>) -> Self {
        self.config.state_script_id = id.into();
        self
    }

    /// Enable debug mode.
    pub fn with_debug(mut self) -> Self {
        self.config.debug = true;
        self
    }

    /// Get the configuration.
    pub fn config(&self) -> &HydrationConfig {
        &self.config
    }
}

impl Default for FullHydration {
    fn default() -> Self {
        Self::new()
    }
}

impl HydrationStrategy for FullHydration {
    fn strategy_type(&self) -> &str {
        "full"
    }

    fn generate_markers(&self, component_id: &str) -> HydrationMarkers {
        HydrationMarkers::hydrated(component_id)
    }

    fn generate_hydration_script(&self, _config: &HydrationConfig) -> String {
        format!(
            r#"<script type="module">
// Ferric Full Hydration Script
(function() {{
    const config = {{
        rootId: '{}',
        stateScriptId: '{}',
        debug: {},
        timeoutMs: {}
    }};

    function log(...args) {{
        if (config.debug) {{
            console.log('[Ferric Hydration]', ...args);
        }}
    }}

    function hydrate() {{
        log('Starting full hydration...');

        const root = document.getElementById(config.rootId);
        if (!root) {{
            console.error('Hydration failed: Root element not found:', config.rootId);
            return;
        }}

        // Load serialized state
        const stateScript = document.getElementById(config.stateScriptId);
        let state = {{}};
        if (stateScript) {{
            try {{
                state = JSON.parse(stateScript.textContent || '{{}}');
                log('Loaded state:', state);
            }} catch (e) {{
                console.error('Failed to parse state:', e);
            }}
        }}

        // Find all components marked for hydration
        const components = root.querySelectorAll('[data-ferric-hydrate="true"]');
        log('Found', components.length, 'components to hydrate');

        components.forEach((el, index) => {{
            const componentId = el.getAttribute('data-ferric-id');
            log('Hydrating component:', componentId);

            // Attach event listeners and restore state
            // This will be handled by the Ferric WASM module
            if (window.__FERRIC__) {{
                window.__FERRIC__.hydrateComponent(el, componentId, state[componentId]);
            }}
        }});

        log('Full hydration complete');

        // Dispatch hydration complete event
        window.dispatchEvent(new CustomEvent('ferric:hydrated', {{
            detail: {{ strategy: 'full', componentCount: components.length }}
        }}));
    }}

    // Wait for WASM module to load
    if (window.__FERRIC__) {{
        hydrate();
    }} else {{
        window.addEventListener('ferric:ready', hydrate);
    }}

    // Timeout fallback
    setTimeout(() => {{
        if (!window.__FERRIC__) {{
            console.warn('Hydration timeout: WASM module not loaded');
        }}
    }}, config.timeoutMs);
}})();
</script>"#,
            self.config.root_id,
            self.config.state_script_id,
            self.config.debug,
            self.config.timeout_ms
        )
    }

    fn should_hydrate(&self, _component_id: &str) -> bool {
        true // Full hydration hydrates everything
    }
}

/// Priority for island hydration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IslandPriority {
    /// No hydration (static content).
    None,
    /// Low priority - hydrate when idle.
    Low,
    /// Normal priority - hydrate on interaction or viewport.
    Normal,
    /// High priority - hydrate immediately.
    High,
    /// Critical priority - hydrate before any other content.
    Critical,
}

impl IslandPriority {
    /// Get the priority as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            IslandPriority::None => "none",
            IslandPriority::Low => "low",
            IslandPriority::Normal => "normal",
            IslandPriority::High => "high",
            IslandPriority::Critical => "critical",
        }
    }
}

/// Loading strategy for islands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IslandLoading {
    /// Hydrate immediately on page load.
    Eager,
    /// Hydrate when the element is visible in viewport.
    Visible,
    /// Hydrate on user interaction (click, focus, etc).
    Interaction,
    /// Hydrate when the browser is idle.
    Idle,
    /// Hydrate on a media query match.
    Media,
}

impl IslandLoading {
    /// Get the loading strategy as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            IslandLoading::Eager => "eager",
            IslandLoading::Visible => "visible",
            IslandLoading::Interaction => "interaction",
            IslandLoading::Idle => "idle",
            IslandLoading::Media => "media",
        }
    }
}

/// Configuration for an island (interactive component).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Island {
    /// Component ID or selector.
    pub id: String,

    /// Hydration priority.
    pub priority: IslandPriority,

    /// Loading strategy.
    pub loading: IslandLoading,

    /// Media query for media-based loading.
    pub media_query: Option<String>,

    /// Props to pass to the component.
    pub props: HashMap<String, serde_json::Value>,
}

impl Island {
    /// Create a new island configuration.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            priority: IslandPriority::Normal,
            loading: IslandLoading::Visible,
            media_query: None,
            props: HashMap::new(),
        }
    }

    /// Set the hydration priority.
    pub fn with_priority(mut self, priority: IslandPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the loading strategy.
    pub fn with_loading(mut self, loading: IslandLoading) -> Self {
        self.loading = loading;
        self
    }

    /// Set a media query for media-based loading.
    pub fn with_media_query(mut self, query: impl Into<String>) -> Self {
        self.media_query = Some(query.into());
        self.loading = IslandLoading::Media;
        self
    }

    /// Add a prop.
    pub fn with_prop(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.props.insert(key.into(), value);
        self
    }
}

/// Partial hydration strategy - selectively hydrates interactive components (islands).
#[derive(Debug, Clone)]
pub struct PartialHydration {
    config: HydrationConfig,
    islands: Vec<Island>,
    island_ids: HashSet<String>,
}

impl PartialHydration {
    /// Create a new partial hydration strategy.
    pub fn new() -> Self {
        Self {
            config: HydrationConfig::default(),
            islands: Vec::new(),
            island_ids: HashSet::new(),
        }
    }

    /// Configure the hydration.
    pub fn with_config(mut self, config: HydrationConfig) -> Self {
        self.config = config;
        self
    }

    /// Add an island to hydrate.
    pub fn add_island(mut self, island: Island) -> Self {
        self.island_ids.insert(island.id.clone());
        self.islands.push(island);
        self
    }

    /// Add multiple islands.
    pub fn with_islands(mut self, islands: Vec<Island>) -> Self {
        for island in islands {
            self.island_ids.insert(island.id.clone());
            self.islands.push(island);
        }
        self
    }

    /// Get all configured islands.
    pub fn islands(&self) -> &[Island] {
        &self.islands
    }

    /// Get the configuration.
    pub fn config(&self) -> &HydrationConfig {
        &self.config
    }
}

impl Default for PartialHydration {
    fn default() -> Self {
        Self::new()
    }
}

impl HydrationStrategy for PartialHydration {
    fn strategy_type(&self) -> &str {
        "partial"
    }

    fn generate_markers(&self, component_id: &str) -> HydrationMarkers {
        if let Some(island) = self.islands.iter().find(|i| i.id == component_id) {
            HydrationMarkers::hydrated(component_id)
                .with_priority(island.priority)
        } else {
            HydrationMarkers::static_component()
        }
    }

    fn generate_hydration_script(&self, _config: &HydrationConfig) -> String {
        let islands_json = serde_json::to_string(&self.islands)
            .unwrap_or_else(|_| "[]".to_string());

        format!(
            r#"<script type="module">
// Ferric Partial Hydration Script (Islands Architecture)
(function() {{
    const config = {{
        rootId: '{}',
        stateScriptId: '{}',
        debug: {},
        timeoutMs: {},
        islands: {}
    }};

    function log(...args) {{
        if (config.debug) {{
            console.log('[Ferric Islands]', ...args);
        }}
    }}

    // Island hydration queue sorted by priority
    const hydratedIslands = new Set();
    const pendingIslands = new Map();

    function loadState() {{
        const stateScript = document.getElementById(config.stateScriptId);
        if (!stateScript) return {{}};

        try {{
            return JSON.parse(stateScript.textContent || '{{}}');
        }} catch (e) {{
            console.error('Failed to parse state:', e);
            return {{}};
        }}
    }}

    function hydrateIsland(element, island, state) {{
        if (hydratedIslands.has(island.id)) {{
            log('Island already hydrated:', island.id);
            return;
        }}

        log('Hydrating island:', island.id, 'priority:', island.priority);

        if (window.__FERRIC__) {{
            window.__FERRIC__.hydrateComponent(
                element,
                island.id,
                state[island.id],
                island.props
            );
            hydratedIslands.add(island.id);

            element.setAttribute('data-ferric-hydrated', 'true');
            element.dispatchEvent(new CustomEvent('ferric:island-hydrated', {{
                detail: {{ islandId: island.id }}
            }}));
        }} else {{
            log('WASM not ready, queuing island:', island.id);
            pendingIslands.set(island.id, {{ element, island, state }});
        }}
    }}

    function setupIntersectionObserver(element, island, state) {{
        const observer = new IntersectionObserver((entries) => {{
            entries.forEach(entry => {{
                if (entry.isIntersecting) {{
                    hydrateIsland(element, island, state);
                    observer.disconnect();
                }}
            }});
        }}, {{ threshold: 0.1 }});

        observer.observe(element);
    }}

    function setupInteractionObserver(element, island, state) {{
        const events = ['click', 'touchstart', 'mouseenter', 'focus'];
        const handler = () => {{
            hydrateIsland(element, island, state);
            events.forEach(ev => element.removeEventListener(ev, handler));
        }};

        events.forEach(ev => element.addEventListener(ev, handler, {{ once: true }}));
    }}

    function setupMediaObserver(element, island, state) {{
        if (!island.media_query) return;

        const mediaQuery = window.matchMedia(island.media_query);
        const checkMedia = () => {{
            if (mediaQuery.matches) {{
                hydrateIsland(element, island, state);
            }}
        }};

        checkMedia();
        mediaQuery.addEventListener('change', checkMedia);
    }}

    function scheduleIsland(element, island, state) {{
        switch (island.loading) {{
            case 'eager':
                hydrateIsland(element, island, state);
                break;
            case 'visible':
                setupIntersectionObserver(element, island, state);
                break;
            case 'interaction':
                setupInteractionObserver(element, island, state);
                break;
            case 'idle':
                if ('requestIdleCallback' in window) {{
                    requestIdleCallback(() => hydrateIsland(element, island, state));
                }} else {{
                    setTimeout(() => hydrateIsland(element, island, state), 1);
                }}
                break;
            case 'media':
                setupMediaObserver(element, island, state);
                break;
        }}
    }}

    function initializeIslands() {{
        log('Initializing islands architecture...');

        const root = document.getElementById(config.rootId);
        if (!root) {{
            console.error('Root element not found:', config.rootId);
            return;
        }}

        const state = loadState();
        log('Loaded state for islands');

        // Sort islands by priority
        const sortedIslands = config.islands.sort((a, b) => {{
            const priorityOrder = {{ critical: 5, high: 4, normal: 3, low: 2, none: 1 }};
            return (priorityOrder[b.priority] || 0) - (priorityOrder[a.priority] || 0);
        }});

        // Find and schedule each island
        sortedIslands.forEach(island => {{
            const elements = root.querySelectorAll(`[data-ferric-id="${{island.id}}"]`);
            log('Found', elements.length, 'elements for island:', island.id);

            elements.forEach(element => {{
                scheduleIsland(element, island, state);
            }});
        }});

        log('Islands initialized, hydration scheduled');

        window.dispatchEvent(new CustomEvent('ferric:islands-ready', {{
            detail: {{ islandCount: sortedIslands.length }}
        }}));
    }}

    // Handle WASM module loading
    function onWasmReady() {{
        log('WASM module ready, processing pending islands...');

        // Hydrate any pending islands
        pendingIslands.forEach(({{ element, island, state }}, id) => {{
            hydrateIsland(element, island, state);
        }});
        pendingIslands.clear();
    }}

    if (window.__FERRIC__) {{
        initializeIslands();
    }} else {{
        window.addEventListener('ferric:ready', () => {{
            onWasmReady();
            initializeIslands();
        }});

        setTimeout(() => {{
            if (!window.__FERRIC__) {{
                console.warn('WASM module not loaded within timeout');
            }}
        }}, config.timeoutMs);
    }}
}})();
</script>"#,
            self.config.root_id,
            self.config.state_script_id,
            self.config.debug,
            self.config.timeout_ms,
            islands_json
        )
    }

    fn should_hydrate(&self, component_id: &str) -> bool {
        self.island_ids.contains(component_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_hydration_markers() {
        let strategy = FullHydration::new();
        let markers = strategy.generate_markers("test-component");

        assert!(markers.hydrate);
        assert!(markers.attributes.contains_key("data-ferric-hydrate"));
        assert!(markers.attributes.contains_key("data-ferric-id"));
    }

    #[test]
    fn test_partial_hydration_should_hydrate() {
        let strategy = PartialHydration::new()
            .add_island(Island::new("interactive-button"))
            .add_island(Island::new("comment-form"));

        assert!(strategy.should_hydrate("interactive-button"));
        assert!(strategy.should_hydrate("comment-form"));
        assert!(!strategy.should_hydrate("static-content"));
    }

    #[test]
    fn test_island_priority_ordering() {
        assert!(IslandPriority::Critical > IslandPriority::High);
        assert!(IslandPriority::High > IslandPriority::Normal);
        assert!(IslandPriority::Normal > IslandPriority::Low);
        assert!(IslandPriority::Low > IslandPriority::None);
    }

    #[test]
    fn test_island_configuration() {
        let island = Island::new("test")
            .with_priority(IslandPriority::High)
            .with_loading(IslandLoading::Visible)
            .with_media_query("(min-width: 768px)")
            .with_prop("count", serde_json::json!(42));

        assert_eq!(island.id, "test");
        assert_eq!(island.priority, IslandPriority::High);
        assert_eq!(island.loading, IslandLoading::Media);
        assert!(island.media_query.is_some());
        assert!(island.props.contains_key("count"));
    }

    #[test]
    fn test_hydration_markers_html_attrs() {
        let markers = HydrationMarkers::hydrated("my-component")
            .with_priority(IslandPriority::High);

        let html = markers.to_html_attrs();
        assert!(html.contains("data-ferric-hydrate=\"true\""));
        assert!(html.contains("data-ferric-id=\"my-component\""));
        assert!(html.contains("data-ferric-priority=\"high\""));
    }
}

