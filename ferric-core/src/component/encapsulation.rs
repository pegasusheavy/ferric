//! View encapsulation implementation.
//!
//! View encapsulation controls how component styles are scoped to prevent
//! them from affecting other parts of the application.
//!
//! ## Encapsulation Modes
//!
//! ### Emulated (Default)
//!
//! Emulates Shadow DOM by adding unique attributes to component elements
//! and rewriting CSS selectors to include those attributes.
//!
//! ```html
//! <!-- Original template -->
//! <div class="container">
//!     <button>Click me</button>
//! </div>
//!
//! <!-- After encapsulation -->
//! <div class="container" _fecontent-abc123>
//!     <button _fecontent-abc123>Click me</button>
//! </div>
//! ```
//!
//! Styles are rewritten:
//! ```css
//! /* Original */
//! .container { padding: 1rem; }
//! button { color: blue; }
//!
//! /* After encapsulation */
//! .container[_fecontent-abc123] { padding: 1rem; }
//! button[_fecontent-abc123] { color: blue; }
//! ```
//!
//! ### ShadowDom
//!
//! Uses native Shadow DOM for true style isolation. Styles are completely
//! isolated and don't leak out or in.
//!
//! ### None
//!
//! No encapsulation. Styles are added globally and can affect any element
//! in the application.

use super::metadata::ViewEncapsulation;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique component IDs.
static COMPONENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique component ID for style scoping.
pub fn generate_component_id() -> String {
    let id = COMPONENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("c{}", id)
}

/// Content attribute name for scoped styles.
pub const CONTENT_ATTR_PREFIX: &str = "_fecontent-";

/// Host attribute name for component host elements.
pub const HOST_ATTR_PREFIX: &str = "_fehost-";

/// Configuration for style encapsulation.
#[derive(Debug, Clone)]
pub struct EncapsulationConfig {
    /// The encapsulation mode.
    pub mode: ViewEncapsulation,
    /// Unique component ID.
    pub component_id: String,
    /// Content attribute (e.g., "_ngcontent-abc123").
    pub content_attr: String,
    /// Host attribute (e.g., "_nghost-abc123").
    pub host_attr: String,
}

impl EncapsulationConfig {
    /// Create a new encapsulation config.
    pub fn new(mode: ViewEncapsulation) -> Self {
        let component_id = generate_component_id();
        let content_attr = format!("{}{}", CONTENT_ATTR_PREFIX, component_id);
        let host_attr = format!("{}{}", HOST_ATTR_PREFIX, component_id);

        Self {
            mode,
            component_id,
            content_attr,
            host_attr,
        }
    }

    /// Create config for no encapsulation.
    pub fn none() -> Self {
        Self {
            mode: ViewEncapsulation::None,
            component_id: String::new(),
            content_attr: String::new(),
            host_attr: String::new(),
        }
    }

    /// Check if this config uses encapsulation.
    pub fn is_encapsulated(&self) -> bool {
        !matches!(self.mode, ViewEncapsulation::None)
    }
}

/// Style encapsulator that transforms CSS for component scoping.
#[derive(Debug)]
pub struct StyleEncapsulator {
    config: EncapsulationConfig,
}

impl StyleEncapsulator {
    /// Create a new style encapsulator.
    pub fn new(config: EncapsulationConfig) -> Self {
        Self { config }
    }

    /// Encapsulate CSS styles based on the encapsulation mode.
    pub fn encapsulate_styles(&self, css: &str) -> String {
        match self.config.mode {
            ViewEncapsulation::Emulated => self.emulate_encapsulation(css),
            ViewEncapsulation::ShadowDom => css.to_string(), // No transformation needed
            ViewEncapsulation::None => css.to_string(),
        }
    }

    /// Transform CSS for emulated encapsulation.
    fn emulate_encapsulation(&self, css: &str) -> String {
        let mut result = String::new();
        let mut chars = css.chars().peekable();
        let content_attr = &self.config.content_attr;

        while let Some(c) = chars.next() {
            match c {
                // Skip comments
                '/' if chars.peek() == Some(&'*') => {
                    result.push(c);
                    result.push(chars.next().unwrap());
                    let mut prev = ' ';
                    for c in chars.by_ref() {
                        result.push(c);
                        if prev == '*' && c == '/' {
                            break;
                        }
                        prev = c;
                    }
                }

                // Skip strings
                '"' | '\'' => {
                    let quote = c;
                    result.push(c);
                    while let Some(c) = chars.next() {
                        result.push(c);
                        if c == quote {
                            break;
                        }
                        if c == '\\'
                            && let Some(escaped) = chars.next() {
                                result.push(escaped);
                            }
                    }
                }

                // Handle rule blocks
                '{' => {
                    result.push(c);
                    // Skip until closing brace (handle nesting)
                    let mut depth = 1;
                    while depth > 0 {
                        if let Some(c) = chars.next() {
                            result.push(c);
                            match c {
                                '{' => depth += 1,
                                '}' => depth -= 1,
                                _ => {}
                            }
                        } else {
                            break;
                        }
                    }
                }

                // Handle selectors
                _ if !c.is_whitespace() && c != '}' && c != '@' => {
                    let mut selector = String::new();
                    selector.push(c);

                    // Read the selector
                    while let Some(&next) = chars.peek() {
                        if next == '{' || next == ',' {
                            break;
                        }
                        selector.push(chars.next().unwrap());
                    }

                    // Transform the selector
                    let transformed = self.transform_selector(&selector, content_attr);
                    result.push_str(&transformed);
                }

                // Pass through at-rules
                '@' => {
                    result.push(c);
                    // Read the at-rule name
                    while let Some(&next) = chars.peek() {
                        if next.is_whitespace() || next == '{' || next == ';' {
                            break;
                        }
                        result.push(chars.next().unwrap());
                    }
                }

                _ => result.push(c),
            }
        }

        result
    }

    /// Transform a single CSS selector for encapsulation.
    fn transform_selector(&self, selector: &str, content_attr: &str) -> String {
        let selector = selector.trim();

        if selector.is_empty() {
            return String::new();
        }

        // Handle :host selector
        if selector.starts_with(":host") {
            return self.transform_host_selector(selector);
        }

        // Handle ::fe-deep (and legacy ::ng-deep for compatibility)
        if selector.contains("::fe-deep") || selector.contains("::ng-deep") || selector.contains("/deep/") || selector.contains(">>>") {
            return selector
                .replace("::fe-deep", "")
                .replace("::ng-deep", "")
                .replace("/deep/", "")
                .replace(">>>", "")
                .trim()
                .to_string();
        }

        // Split compound selectors and transform each part
        let parts: Vec<&str> = selector.split(',').collect();
        let transformed: Vec<String> = parts
            .iter()
            .map(|part| self.transform_simple_selector(part.trim(), content_attr))
            .collect();

        transformed.join(", ")
    }

    /// Transform a simple selector (non-compound).
    fn transform_simple_selector(&self, selector: &str, content_attr: &str) -> String {
        if selector.is_empty() {
            return String::new();
        }

        // Find where to insert the attribute selector
        // We want to insert after the element/class/id but before pseudo-elements
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = selector.chars().peekable();
        let mut in_brackets = 0;

        for c in chars {
            match c {
                '[' => {
                    in_brackets += 1;
                    current.push(c);
                }
                ']' => {
                    in_brackets -= 1;
                    current.push(c);
                }
                ' ' if in_brackets == 0 => {
                    if !current.is_empty() {
                        parts.push(current);
                        current = String::new();
                    }
                    parts.push(" ".to_string());
                }
                '>' | '+' | '~' if in_brackets == 0 => {
                    if !current.is_empty() {
                        parts.push(current);
                        current = String::new();
                    }
                    parts.push(format!(" {} ", c));
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        // Add content attribute to each element part
        let transformed: Vec<String> = parts
            .iter()
            .map(|part| {
                let part = part.trim();
                if part.is_empty() || part == ">" || part == "+" || part == "~" {
                    part.to_string()
                } else if part.starts_with("::") {
                    // Pseudo-element - don't add attribute
                    part.to_string()
                } else if part.starts_with(':') && !part.starts_with("::") {
                    // Pseudo-class - insert attribute before it
                    format!("[{}]{}", content_attr, part)
                } else if part.contains("::") {
                    // Element with pseudo-element
                    let idx = part.find("::").unwrap();
                    format!("{}[{}]{}", &part[..idx], content_attr, &part[idx..])
                } else if part.contains(':') {
                    // Element with pseudo-class
                    let idx = part.find(':').unwrap();
                    format!("{}[{}]{}", &part[..idx], content_attr, &part[idx..])
                } else {
                    format!("{}[{}]", part, content_attr)
                }
            })
            .collect();

        transformed.join("")
    }

    /// Transform :host selectors.
    fn transform_host_selector(&self, selector: &str) -> String {
        let host_attr = &self.config.host_attr;

        if selector == ":host" {
            return format!("[{}]", host_attr);
        }

        // :host(.class) or :host([attr])
        if selector.starts_with(":host(") && selector.ends_with(')') {
            let inner = &selector[6..selector.len() - 1];
            return format!("[{}]{}", host_attr, inner);
        }

        // :host-context(.class)
        if selector.starts_with(":host-context(") && selector.ends_with(')') {
            let inner = &selector[14..selector.len() - 1];
            return format!("{} [{}]", inner, host_attr);
        }

        selector.to_string()
    }

    /// Get the content attribute for this encapsulator.
    pub fn content_attr(&self) -> &str {
        &self.config.content_attr
    }

    /// Get the host attribute for this encapsulator.
    pub fn host_attr(&self) -> &str {
        &self.config.host_attr
    }
}

/// Template encapsulator that adds scope attributes to HTML elements.
#[derive(Debug)]
pub struct TemplateEncapsulator {
    config: EncapsulationConfig,
}

impl TemplateEncapsulator {
    /// Create a new template encapsulator.
    pub fn new(config: EncapsulationConfig) -> Self {
        Self { config }
    }

    /// Encapsulate an HTML template.
    pub fn encapsulate_template(&self, html: &str) -> String {
        match self.config.mode {
            ViewEncapsulation::Emulated => self.add_content_attributes(html),
            ViewEncapsulation::ShadowDom => html.to_string(),
            ViewEncapsulation::None => html.to_string(),
        }
    }

    /// Add content attributes to all elements in the template.
    fn add_content_attributes(&self, html: &str) -> String {
        let content_attr = &self.config.content_attr;
        let mut result = String::new();
        let mut chars = html.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '<' {
                // Check if it's a tag
                if let Some(&next) = chars.peek() {
                    if next == '/' || next == '!' {
                        // Closing tag or comment
                        result.push(c);
                    } else if next.is_alphabetic() {
                        // Opening tag
                        result.push(c);
                        let tag_result = self.process_tag(&mut chars, content_attr);
                        result.push_str(&tag_result);
                        continue;
                    } else {
                        result.push(c);
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Process a single HTML tag, adding the content attribute.
    fn process_tag(&self, chars: &mut std::iter::Peekable<std::str::Chars>, content_attr: &str) -> String {
        let mut tag_content = String::new();
        let mut tag_name = String::new();
        let mut in_tag_name = true;

        while let Some(c) = chars.next() {
            if in_tag_name {
                if c.is_whitespace() || c == '>' || c == '/' {
                    in_tag_name = false;

                    // Skip certain tags that shouldn't have attributes
                    let skip_tags = ["fe-content", "fe-container", "fe-template"];
                    if skip_tags.contains(&tag_name.to_lowercase().as_str()) {
                        tag_content.push(c);
                        // Just pass through the rest
                        for c in chars.by_ref() {
                            tag_content.push(c);
                            if c == '>' {
                                break;
                            }
                        }
                        return format!("{}{}", tag_name, tag_content);
                    }

                    if c == '>' {
                        // Self-contained tag with no attributes - add attribute before >
                        return format!("{} {}>", tag_name, content_attr);
                    } else if c == '/' {
                        // Could be self-closing
                        if chars.peek() == Some(&'>') {
                            chars.next();
                            return format!("{} {}/>", tag_name, content_attr);
                        }
                    }

                    tag_content.push(c);
                } else {
                    tag_name.push(c);
                }
            } else {
                if c == '>' {
                    // Check if we already have attributes
                    let trimmed = tag_content.trim();
                    if trimmed.ends_with('/') {
                        // Self-closing with attributes
                        let without_slash = tag_content.trim_end().strip_suffix('/').unwrap_or(&tag_content);
                        return format!("{}{} {}/>", tag_name, without_slash, content_attr);
                    } else {
                        return format!("{}{} {}>", tag_name, tag_content, content_attr);
                    }
                }
                tag_content.push(c);
            }
        }

        format!("{}{}", tag_name, tag_content)
    }

    /// Get the content attribute.
    pub fn content_attr(&self) -> &str {
        &self.config.content_attr
    }

    /// Get the host attribute.
    pub fn host_attr(&self) -> &str {
        &self.config.host_attr
    }
}

/// Combined encapsulator for both styles and templates.
#[derive(Debug)]
pub struct ViewEncapsulator {
    /// Style encapsulator.
    pub style: StyleEncapsulator,
    /// Template encapsulator.
    pub template: TemplateEncapsulator,
    /// Configuration.
    config: EncapsulationConfig,
}

impl ViewEncapsulator {
    /// Create a new view encapsulator.
    pub fn new(mode: ViewEncapsulation) -> Self {
        let config = EncapsulationConfig::new(mode);
        Self {
            style: StyleEncapsulator::new(config.clone()),
            template: TemplateEncapsulator::new(config.clone()),
            config,
        }
    }

    /// Create for no encapsulation.
    pub fn none() -> Self {
        let config = EncapsulationConfig::none();
        Self {
            style: StyleEncapsulator::new(config.clone()),
            template: TemplateEncapsulator::new(config.clone()),
            config,
        }
    }

    /// Encapsulate both styles and template.
    pub fn encapsulate(&self, template: &str, styles: &str) -> (String, String) {
        let encapsulated_template = self.template.encapsulate_template(template);
        let encapsulated_styles = self.style.encapsulate_styles(styles);
        (encapsulated_template, encapsulated_styles)
    }

    /// Get the encapsulation mode.
    pub fn mode(&self) -> ViewEncapsulation {
        self.config.mode
    }

    /// Get the component ID.
    pub fn component_id(&self) -> &str {
        &self.config.component_id
    }

    /// Get the content attribute.
    pub fn content_attr(&self) -> &str {
        &self.config.content_attr
    }

    /// Get the host attribute.
    pub fn host_attr(&self) -> &str {
        &self.config.host_attr
    }
}

/// Cache for encapsulated styles to avoid re-processing.
thread_local! {
    static STYLE_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Get cached encapsulated styles or compute and cache them.
pub fn get_or_encapsulate_styles(
    component_id: &str,
    original_styles: &str,
    mode: ViewEncapsulation,
) -> String {
    let cache_key = format!("{}:{:?}", component_id, mode);

    STYLE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }

        let config = EncapsulationConfig {
            mode,
            component_id: component_id.to_string(),
            content_attr: format!("{}{}", CONTENT_ATTR_PREFIX, component_id),
            host_attr: format!("{}{}", HOST_ATTR_PREFIX, component_id),
        };

        let encapsulator = StyleEncapsulator::new(config);
        let encapsulated = encapsulator.encapsulate_styles(original_styles);

        cache.insert(cache_key, encapsulated.clone());
        encapsulated
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_encapsulation_simple() {
        let config = EncapsulationConfig {
            mode: ViewEncapsulation::Emulated,
            component_id: "test".to_string(),
            content_attr: "_fecontent-test".to_string(),
            host_attr: "_fehost-test".to_string(),
        };

        let encapsulator = StyleEncapsulator::new(config);
        let result = encapsulator.encapsulate_styles(".container { color: red; }");

        assert!(result.contains("[_fecontent-test]"));
    }

    #[test]
    fn test_style_encapsulation_host() {
        let config = EncapsulationConfig {
            mode: ViewEncapsulation::Emulated,
            component_id: "test".to_string(),
            content_attr: "_fecontent-test".to_string(),
            host_attr: "_fehost-test".to_string(),
        };

        let encapsulator = StyleEncapsulator::new(config);
        let result = encapsulator.encapsulate_styles(":host { display: block; }");

        assert!(result.contains("[_fehost-test]"));
    }

    #[test]
    fn test_template_encapsulation() {
        let config = EncapsulationConfig {
            mode: ViewEncapsulation::Emulated,
            component_id: "test".to_string(),
            content_attr: "_fecontent-test".to_string(),
            host_attr: "_fehost-test".to_string(),
        };

        let encapsulator = TemplateEncapsulator::new(config);
        let result = encapsulator.encapsulate_template("<div class=\"container\"><span>Hello</span></div>");

        assert!(result.contains("_fecontent-test"));
    }

    #[test]
    fn test_no_encapsulation() {
        let encapsulator = ViewEncapsulator::none();
        let (template, styles) = encapsulator.encapsulate(
            "<div>Hello</div>",
            ".container { color: red; }",
        );

        assert_eq!(template, "<div>Hello</div>");
        assert_eq!(styles, ".container { color: red; }");
    }

    #[test]
    fn test_self_closing_tags() {
        let config = EncapsulationConfig {
            mode: ViewEncapsulation::Emulated,
            component_id: "test".to_string(),
            content_attr: "_fecontent-test".to_string(),
            host_attr: "_fehost-test".to_string(),
        };

        let encapsulator = TemplateEncapsulator::new(config);
        let result = encapsulator.encapsulate_template("<input type=\"text\" />");

        assert!(result.contains("_fecontent-test"));
    }

    #[test]
    fn test_fe_content_not_modified() {
        let config = EncapsulationConfig {
            mode: ViewEncapsulation::Emulated,
            component_id: "test".to_string(),
            content_attr: "_fecontent-test".to_string(),
            host_attr: "_fehost-test".to_string(),
        };

        let encapsulator = TemplateEncapsulator::new(config);
        let result = encapsulator.encapsulate_template("<fe-content select=\".header\"></fe-content>");

        // fe-content should not have the attribute added
        assert!(!result.contains("fe-content _fecontent"));
    }

    #[test]
    fn test_combined_encapsulator() {
        let encapsulator = ViewEncapsulator::new(ViewEncapsulation::Emulated);

        let (template, styles) = encapsulator.encapsulate(
            "<div class=\"card\"><p>Content</p></div>",
            ".card { padding: 1rem; } p { margin: 0; }",
        );

        let content_attr = encapsulator.content_attr();
        assert!(template.contains(content_attr));
        assert!(styles.contains(&format!("[{}]", content_attr)));
    }
}
