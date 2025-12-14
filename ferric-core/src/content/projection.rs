//! Content projection implementation.

use super::selector::{matches_selector, ContentSelector};
use super::slot::{find_slots, replace_slots, ContentSlot, SlotRef};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Represents content to be projected into a component.
#[derive(Debug, Clone)]
pub struct ProjectedContent {
    /// The HTML content.
    pub html: String,
    /// Tag name of the content element.
    pub tag_name: String,
    /// CSS classes on the content element.
    pub classes: Vec<String>,
    /// Attributes on the content element.
    pub attributes: Vec<(String, Option<String>)>,
}

impl ProjectedContent {
    /// Create projected content from HTML.
    pub fn from_html(html: impl Into<String>) -> Self {
        let html = html.into();
        Self {
            html,
            tag_name: String::new(),
            classes: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Create projected content with metadata.
    pub fn new(
        html: impl Into<String>,
        tag_name: impl Into<String>,
        classes: Vec<String>,
        attributes: Vec<(String, Option<String>)>,
    ) -> Self {
        Self {
            html: html.into(),
            tag_name: tag_name.into(),
            classes,
            attributes,
        }
    }

    /// Check if this content matches a selector.
    pub fn matches(&self, selector: &ContentSelector) -> bool {
        if selector.is_default() {
            return true;
        }

        let class_refs: Vec<&str> = self.classes.iter().map(|s| s.as_str()).collect();
        let attr_refs: Vec<(&str, Option<&str>)> = self
            .attributes
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_ref().map(|s| s.as_str())))
            .collect();

        matches_selector(&selector.kind, &self.tag_name, &class_refs, &attr_refs)
    }
}

/// Context for content projection during component rendering.
#[derive(Debug, Clone, Default)]
pub struct ProjectionContext {
    /// Content items to be projected.
    content: Vec<ProjectedContent>,
    /// Cached slot assignments.
    assignments: RefCell<Option<HashMap<SlotRef, String>>>,
}

impl ProjectionContext {
    /// Create a new projection context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add content to be projected.
    pub fn add(&mut self, content: ProjectedContent) {
        self.content.push(content);
        // Clear cached assignments
        *self.assignments.borrow_mut() = None;
    }

    /// Add multiple content items.
    pub fn add_all(&mut self, content: impl IntoIterator<Item = ProjectedContent>) {
        self.content.extend(content);
        *self.assignments.borrow_mut() = None;
    }

    /// Check if there's any content to project.
    pub fn has_content(&self) -> bool {
        !self.content.is_empty()
    }

    /// Get the number of content items.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Assign content to slots.
    fn compute_assignments(&self, slots: &[ContentSlot]) -> HashMap<SlotRef, String> {
        let mut assignments: HashMap<SlotRef, String> = HashMap::new();
        let mut used_content: Vec<bool> = vec![false; self.content.len()];

        // First pass: match content to specific selectors (non-default slots)
        for slot in slots.iter().filter(|s| !s.is_default()) {
            let mut slot_content = Vec::new();

            for (i, content) in self.content.iter().enumerate() {
                if !used_content[i] && content.matches(&slot.selector) {
                    slot_content.push(content.html.clone());
                    used_content[i] = true;
                }
            }

            if !slot_content.is_empty() {
                assignments.insert(SlotRef(slot.id), slot_content.join("\n"));
            }
        }

        // Second pass: remaining content goes to default slot
        if let Some(default_slot) = slots.iter().find(|s| s.is_default()) {
            let remaining: Vec<String> = self
                .content
                .iter()
                .enumerate()
                .filter(|(i, _)| !used_content[*i])
                .map(|(_, c)| c.html.clone())
                .collect();

            if !remaining.is_empty() {
                assignments.insert(SlotRef(default_slot.id), remaining.join("\n"));
            }
        }

        assignments
    }

    /// Project content into a template.
    pub fn project(&self, template: &str) -> String {
        let slots = find_slots(template);

        if slots.is_empty() {
            return template.to_string();
        }

        let assignments = self.compute_assignments(&slots);
        replace_slots(template, &slots, &assignments)
    }
}

/// Main content projection handler.
#[derive(Debug, Clone)]
pub struct ContentProjection {
    /// The component's template with slot definitions.
    template: String,
    /// Slots found in the template.
    slots: Vec<ContentSlot>,
    /// Current projection context.
    context: Rc<RefCell<ProjectionContext>>,
}

impl ContentProjection {
    /// Create a new content projection for a template.
    pub fn new(template: impl Into<String>) -> Self {
        let template = template.into();
        let slots = find_slots(&template);

        Self {
            template,
            slots,
            context: Rc::new(RefCell::new(ProjectionContext::new())),
        }
    }

    /// Get a reference to the projection context.
    pub fn context(&self) -> Rc<RefCell<ProjectionContext>> {
        self.context.clone()
    }

    /// Set the content to be projected.
    pub fn set_content(&self, content: Vec<ProjectedContent>) {
        let mut ctx = self.context.borrow_mut();
        ctx.content = content;
        *ctx.assignments.borrow_mut() = None;
    }

    /// Add content to be projected.
    pub fn add_content(&self, content: ProjectedContent) {
        self.context.borrow_mut().add(content);
    }

    /// Check if there are any slots in this template.
    pub fn has_slots(&self) -> bool {
        !self.slots.is_empty()
    }

    /// Get the slots in this template.
    pub fn slots(&self) -> &[ContentSlot] {
        &self.slots
    }

    /// Render the template with projected content.
    pub fn render(&self) -> String {
        self.context.borrow().project(&self.template)
    }

    /// Check if there's a default slot.
    pub fn has_default_slot(&self) -> bool {
        self.slots.iter().any(|s| s.is_default())
    }

    /// Get slot selectors.
    pub fn selectors(&self) -> Vec<&ContentSelector> {
        self.slots.iter().map(|s| &s.selector).collect()
    }
}

/// Convenience function to project content into a template.
pub fn project_content(template: &str, content: &[ProjectedContent]) -> String {
    let ctx = ProjectionContext {
        content: content.to_vec(),
        assignments: RefCell::new(None),
    };
    ctx.project(template)
}

/// Create a projection context from HTML children.
pub fn create_projection_context(children_html: &str) -> ProjectionContext {
    let mut ctx = ProjectionContext::new();

    // Parse children HTML into ProjectedContent items
    // This is a simplified parser - in production, use a proper HTML parser
    let content = parse_children(children_html);
    ctx.add_all(content);

    ctx
}

/// Parse HTML children into ProjectedContent items.
fn parse_children(html: &str) -> Vec<ProjectedContent> {
    let mut content = Vec::new();
    let html = html.trim();

    if html.is_empty() {
        return content;
    }

    // Simple parser for direct child elements
    let mut depth = 0;
    let mut current_start = 0;
    let mut in_tag = false;
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            if i + 1 < chars.len() && chars[i + 1] == '/' {
                // Closing tag
                depth -= 1;
                if depth == 0 {
                    // Find end of this tag
                    if let Some(end) = html[i..].find('>') {
                        let element_html = &html[current_start..=i + end];
                        if let Some(pc) = parse_single_element(element_html) {
                            content.push(pc);
                        }
                        i += end + 1;
                        current_start = i;
                        continue;
                    }
                }
            } else if i + 1 < chars.len() && chars[i + 1] != '!' {
                // Opening tag (not comment)
                if depth == 0 {
                    current_start = i;
                }
                depth += 1;
                in_tag = true;
            }
        } else if chars[i] == '>' && in_tag {
            in_tag = false;
            // Check for self-closing
            if i > 0 && chars[i - 1] == '/' {
                depth -= 1;
                if depth == 0 {
                    let element_html = &html[current_start..=i];
                    if let Some(pc) = parse_single_element(element_html) {
                        content.push(pc);
                    }
                    current_start = i + 1;
                }
            }
        }
        i += 1;
    }

    // Handle remaining text content
    let remaining = html[current_start..].trim();
    if !remaining.is_empty() && depth == 0 {
        content.push(ProjectedContent::from_html(remaining));
    }

    content
}

/// Parse a single HTML element into ProjectedContent.
fn parse_single_element(html: &str) -> Option<ProjectedContent> {
    let html = html.trim();
    if html.is_empty() || !html.starts_with('<') {
        return Some(ProjectedContent::from_html(html));
    }

    // Find tag name
    let tag_end = html[1..]
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .map(|i| i + 1)
        .unwrap_or(1);

    let tag_name = html[1..tag_end].to_lowercase();

    // Parse attributes
    let attr_section_end = html.find('>').unwrap_or(html.len());
    let attr_section = &html[tag_end..attr_section_end];

    let mut classes = Vec::new();
    let mut attributes = Vec::new();

    // Simple attribute parsing
    let mut attr_chars = attr_section.chars().peekable();
    while let Some(&c) = attr_chars.peek() {
        if c.is_whitespace() {
            attr_chars.next();
            continue;
        }

        // Read attribute name
        let mut attr_name = String::new();
        while let Some(&c) = attr_chars.peek() {
            if c == '=' || c.is_whitespace() || c == '>' || c == '/' {
                break;
            }
            attr_name.push(c);
            attr_chars.next();
        }

        if attr_name.is_empty() {
            attr_chars.next();
            continue;
        }

        // Skip whitespace
        while attr_chars.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
            attr_chars.next();
        }

        // Check for value
        let attr_value = if attr_chars.peek() == Some(&'=') {
            attr_chars.next(); // consume '='

            // Skip whitespace
            while attr_chars.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                attr_chars.next();
            }

            // Read value
            let quote = attr_chars.peek().copied();
            if quote == Some('"') || quote == Some('\'') {
                attr_chars.next(); // consume opening quote
                let mut value = String::new();
                while let Some(&c) = attr_chars.peek() {
                    if Some(c) == quote {
                        attr_chars.next();
                        break;
                    }
                    value.push(c);
                    attr_chars.next();
                }
                Some(value)
            } else {
                // Unquoted value
                let mut value = String::new();
                while let Some(&c) = attr_chars.peek() {
                    if c.is_whitespace() || c == '>' || c == '/' {
                        break;
                    }
                    value.push(c);
                    attr_chars.next();
                }
                Some(value)
            }
        } else {
            None
        };

        // Handle class attribute specially
        if attr_name == "class"
            && let Some(ref class_str) = attr_value {
                classes.extend(class_str.split_whitespace().map(|s| s.to_string()));
            }

        attributes.push((attr_name, attr_value));
    }

    Some(ProjectedContent::new(html, tag_name, classes, attributes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_projection_basic() {
        let projection = ContentProjection::new("<div><fe-content></fe-content></div>");

        projection.add_content(ProjectedContent::from_html("<p>Hello</p>"));

        let result = projection.render();
        assert_eq!(result, "<div><p>Hello</p></div>");
    }

    #[test]
    fn test_content_projection_with_selector() {
        let template = r#"
            <header><fe-content select="[header]"></fe-content></header>
            <main><fe-content></fe-content></main>
        "#;

        let projection = ContentProjection::new(template);

        projection.add_content(ProjectedContent::new(
            "<h1>Title</h1>",
            "h1",
            vec![],
            vec![("header".to_string(), None)],
        ));
        projection.add_content(ProjectedContent::from_html("<p>Body</p>"));

        let result = projection.render();
        assert!(result.contains("<header><h1>Title</h1></header>"));
        assert!(result.contains("<main><p>Body</p></main>"));
    }

    #[test]
    fn test_project_content_function() {
        let template = "<div><fe-content></fe-content></div>";
        let content = vec![ProjectedContent::from_html("<span>Test</span>")];

        let result = project_content(template, &content);
        assert_eq!(result, "<div><span>Test</span></div>");
    }

    #[test]
    fn test_multiple_content_to_default() {
        let template = "<fe-content></fe-content>";
        let content = vec![
            ProjectedContent::from_html("<p>One</p>"),
            ProjectedContent::from_html("<p>Two</p>"),
        ];

        let result = project_content(template, &content);
        assert_eq!(result, "<p>One</p>\n<p>Two</p>");
    }

    #[test]
    fn test_class_selector() {
        let template = r#"<fe-content select=".highlight"></fe-content>"#;

        let content = vec![
            ProjectedContent::new("<span>Highlighted</span>", "span", vec!["highlight".to_string()], vec![]),
            ProjectedContent::new("<span>Normal</span>", "span", vec![], vec![]),
        ];

        let result = project_content(template, &content);
        assert_eq!(result, "<span>Highlighted</span>");
    }

    #[test]
    fn test_fallback_content() {
        let template = "<fe-content>Default</fe-content>";
        let content: Vec<ProjectedContent> = vec![];

        let result = project_content(template, &content);
        assert_eq!(result, "Default");
    }
}

