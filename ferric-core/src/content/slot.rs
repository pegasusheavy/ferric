//! Content slot detection and management.

use super::selector::ContentSelector;
use std::collections::HashMap;

/// Represents a content slot (`<fe-content>`) in a template.
#[derive(Debug, Clone)]
pub struct ContentSlot {
    /// Unique identifier for this slot.
    pub id: usize,
    /// The selector for this slot (empty for default).
    pub selector: ContentSelector,
    /// Fallback content if nothing is projected.
    pub fallback: Option<String>,
    /// Position in the original template.
    pub position: SlotPosition,
}

impl ContentSlot {
    /// Create a new content slot.
    pub fn new(id: usize, selector: ContentSelector) -> Self {
        Self {
            id,
            selector,
            fallback: None,
            position: SlotPosition::default(),
        }
    }

    /// Set fallback content.
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }

    /// Set position.
    pub fn with_position(mut self, start: usize, end: usize) -> Self {
        self.position = SlotPosition { start, end };
        self
    }

    /// Check if this is the default slot.
    pub fn is_default(&self) -> bool {
        self.selector.is_default()
    }
}

/// Position of a slot in the template string.
#[derive(Debug, Clone, Copy, Default)]
pub struct SlotPosition {
    /// Start index in the template.
    pub start: usize,
    /// End index in the template.
    pub end: usize,
}

/// A reference to a slot for tracking purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotRef(pub usize);

impl SlotRef {
    /// Create a new slot reference.
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// Find all `<fe-content>` slots in a template.
pub fn find_slots(template: &str) -> Vec<ContentSlot> {
    let mut slots = Vec::new();
    let mut id = 0;
    let mut search_start = 0;

    while let Some(start) = template[search_start..].find("<fe-content") {
        let absolute_start = search_start + start;

        // Find the end of this tag
        let tag_content = &template[absolute_start..];

        // Check for self-closing or with content
        if let Some(end_pos) = find_slot_end(tag_content) {
            let absolute_end = absolute_start + end_pos;
            let tag_str = &template[absolute_start..absolute_end];

            // Parse the slot
            let selector = parse_select_attribute(tag_str);
            let fallback = parse_fallback_content(tag_str);

            let mut slot = ContentSlot::new(id, selector)
                .with_position(absolute_start, absolute_end);

            if let Some(fb) = fallback {
                slot = slot.with_fallback(fb);
            }

            slots.push(slot);
            id += 1;
            search_start = absolute_end;
        } else {
            // Malformed tag, skip
            search_start = absolute_start + 11; // len("<fe-content")
        }
    }

    slots
}

/// Find the end position of a `<fe-content>` tag.
fn find_slot_end(tag_content: &str) -> Option<usize> {
    // Self-closing: <fe-content ... />
    if let Some(pos) = tag_content.find("/>") {
        // Make sure there's no closing tag before this
        if let Some(close_pos) = tag_content.find("</fe-content>")
            && close_pos < pos {
                return Some(close_pos + "</fe-content>".len());
            }
        return Some(pos + 2);
    }

    // With closing tag: <fe-content>...</fe-content>
    if let Some(pos) = tag_content.find("</fe-content>") {
        return Some(pos + "</fe-content>".len());
    }

    // Just opening tag with >: <fe-content>
    if let Some(pos) = tag_content.find('>') {
        // Check if there's a closing tag
        if let Some(close_pos) = tag_content.find("</fe-content>") {
            return Some(close_pos + "</fe-content>".len());
        }
        // Self-contained
        return Some(pos + 1);
    }

    None
}

/// Parse the `select` attribute from a slot tag.
fn parse_select_attribute(tag: &str) -> ContentSelector {
    // Find select="..." or select='...'
    let patterns = ["select=\"", "select='", "select="];

    for pattern in patterns {
        if let Some(start) = tag.find(pattern) {
            let value_start = start + pattern.len();
            let rest = &tag[value_start..];

            // Find the closing quote or whitespace/bracket
            let end_char = if pattern.ends_with('"') {
                '"'
            } else if pattern.ends_with('\'') {
                '\''
            } else {
                ' ' // or > or /
            };

            let value_end = rest.find(end_char)
                .or_else(|| rest.find('>'))
                .or_else(|| rest.find('/'))
                .unwrap_or(rest.len());

            let selector_value = &rest[..value_end];
            return ContentSelector::new(selector_value);
        }
    }

    ContentSelector::default()
}

/// Parse fallback content from between opening and closing tags.
fn parse_fallback_content(tag: &str) -> Option<String> {
    // Find content between > and </fe-content>
    let open_end = tag.find('>')?;

    // Check for self-closing
    if tag[..open_end].ends_with('/') {
        return None;
    }

    let close_start = tag.find("</fe-content>")?;

    if close_start > open_end + 1 {
        let content = &tag[open_end + 1..close_start];
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

/// Replace content slots in a template with projected content.
pub fn replace_slots(
    template: &str,
    slots: &[ContentSlot],
    projected: &HashMap<SlotRef, String>,
) -> String {
    let mut result = template.to_string();

    // Process slots in reverse order to maintain correct positions
    let mut sorted_slots: Vec<_> = slots.iter().collect();
    sorted_slots.sort_by(|a, b| b.position.start.cmp(&a.position.start));

    for slot in sorted_slots {
        let slot_ref = SlotRef(slot.id);
        let replacement = projected
            .get(&slot_ref)
            .cloned()
            .or_else(|| slot.fallback.clone())
            .unwrap_or_default();

        let before = &result[..slot.position.start];
        let after = &result[slot.position.end..];
        result = format!("{}{}{}", before, replacement, after);
    }

    result
}

/// Create a placeholder comment for a slot (used during rendering).
pub fn slot_placeholder(slot_ref: SlotRef) -> String {
    format!("<!--fe-slot:{}-->", slot_ref.0)
}

/// Find slot placeholders in rendered HTML.
pub fn find_slot_placeholders(html: &str) -> Vec<(SlotRef, usize, usize)> {
    let mut results = Vec::new();
    let mut search_start = 0;

    while let Some(start) = html[search_start..].find("<!--fe-slot:") {
        let absolute_start = search_start + start;
        let rest = &html[absolute_start + 12..]; // After "<!--fe-slot:"

        if let Some(end_pos) = rest.find("-->") {
            let id_str = &rest[..end_pos];
            if let Ok(id) = id_str.parse::<usize>() {
                let absolute_end = absolute_start + 12 + end_pos + 3;
                results.push((SlotRef(id), absolute_start, absolute_end));
            }
            search_start = absolute_start + 12 + end_pos + 3;
        } else {
            search_start = absolute_start + 12;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_slots_simple() {
        let template = r#"<div><fe-content></fe-content></div>"#;
        let slots = find_slots(template);
        assert_eq!(slots.len(), 1);
        assert!(slots[0].is_default());
    }

    #[test]
    fn test_find_slots_self_closing() {
        let template = r#"<div><fe-content /></div>"#;
        let slots = find_slots(template);
        assert_eq!(slots.len(), 1);
    }

    #[test]
    fn test_find_slots_with_select() {
        let template = r#"<fe-content select="[header]"></fe-content>"#;
        let slots = find_slots(template);
        assert_eq!(slots.len(), 1);
        assert!(!slots[0].is_default());
        assert_eq!(slots[0].selector.raw, "[header]");
    }

    #[test]
    fn test_find_slots_multiple() {
        let template = r#"
            <header><fe-content select="[header]"></fe-content></header>
            <main><fe-content></fe-content></main>
            <footer><fe-content select=".footer"></fe-content></footer>
        "#;
        let slots = find_slots(template);
        assert_eq!(slots.len(), 3);
    }

    #[test]
    fn test_find_slots_with_fallback() {
        let template = r#"<fe-content select="[title]"><h1>Default Title</h1></fe-content>"#;
        let slots = find_slots(template);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].fallback, Some("<h1>Default Title</h1>".to_string()));
    }

    #[test]
    fn test_replace_slots() {
        let template = r#"<div><fe-content></fe-content></div>"#;
        let slots = find_slots(template);

        let mut projected = HashMap::new();
        projected.insert(SlotRef(0), "<p>Hello</p>".to_string());

        let result = replace_slots(template, &slots, &projected);
        assert_eq!(result, "<div><p>Hello</p></div>");
    }

    #[test]
    fn test_replace_slots_with_fallback() {
        let template = r#"<fe-content>Fallback</fe-content>"#;
        let slots = find_slots(template);

        let projected = HashMap::new(); // Empty - should use fallback

        let result = replace_slots(template, &slots, &projected);
        assert_eq!(result, "Fallback");
    }
}

