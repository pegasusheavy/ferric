//! Component metadata and decorators.

/// Metadata for a component definition.
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    /// The CSS selector used to identify this component in templates.
    pub selector: String,
    /// The component's template.
    pub template: Option<String>,
    /// URL to an external template file.
    pub template_url: Option<String>,
    /// Inline styles for the component.
    pub styles: Vec<String>,
    /// URLs to external stylesheet files.
    pub style_urls: Vec<String>,
    /// Input property bindings.
    pub inputs: Vec<String>,
    /// Output event bindings.
    pub outputs: Vec<String>,
    /// Services to provide at the component level.
    pub providers: Vec<String>,
    /// View encapsulation mode.
    pub encapsulation: ViewEncapsulation,
    /// Change detection strategy.
    pub change_detection: ChangeDetectionStrategy,
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        Self {
            selector: String::new(),
            template: None,
            template_url: None,
            styles: Vec::new(),
            style_urls: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            providers: Vec::new(),
            encapsulation: ViewEncapsulation::Emulated,
            change_detection: ChangeDetectionStrategy::Default,
        }
    }
}

/// View encapsulation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewEncapsulation {
    /// Emulate shadow DOM by adding unique attributes to elements.
    Emulated,
    /// Use native Shadow DOM.
    ShadowDom,
    /// No encapsulation - styles are global.
    None,
}

/// Change detection strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeDetectionStrategy {
    /// Check the component whenever any change might have occurred.
    Default,
    /// Only check when inputs change or events are triggered.
    OnPush,
}

/// Input property metadata.
#[derive(Debug, Clone)]
pub struct InputMetadata {
    /// The property name.
    pub name: String,
    /// Alias for the input binding.
    pub alias: Option<String>,
    /// Whether this input is required.
    pub required: bool,
    /// Transform function name.
    pub transform: Option<String>,
}

/// Output event metadata.
#[derive(Debug, Clone)]
pub struct OutputMetadata {
    /// The property name.
    pub name: String,
    /// Alias for the output binding.
    pub alias: Option<String>,
}

