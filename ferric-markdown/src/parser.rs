//! Markdown parsing using pulldown-cmark.

use pulldown_cmark::{Options, Parser as CmarkParser};

/// Markdown parser wrapper.
pub struct MarkdownParser {
    options: Options,
}

impl MarkdownParser {
    /// Create a new parser with default options.
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        Self { options }
    }

    /// Create a parser with GitHub Flavored Markdown options.
    pub fn github() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        Self { options }
    }

    /// Create a parser with custom options.
    pub fn with_options(
        enable_tables: bool,
        enable_footnotes: bool,
        enable_strikethrough: bool,
        enable_tasklists: bool,
    ) -> Self {
        let mut options = Options::empty();

        if enable_tables {
            options.insert(Options::ENABLE_TABLES);
        }
        if enable_footnotes {
            options.insert(Options::ENABLE_FOOTNOTES);
        }
        if enable_strikethrough {
            options.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if enable_tasklists {
            options.insert(Options::ENABLE_TASKLISTS);
        }

        Self { options }
    }

    /// Parse markdown text into events.
    pub fn parse<'a>(&self, text: &'a str) -> CmarkParser<'a> {
        CmarkParser::new_ext(text, self.options)
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Event;

    #[test]
    fn test_basic_parsing() {
        let parser = MarkdownParser::new();
        let events: Vec<Event> = parser.parse("# Hello").collect();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_table_parsing() {
        let parser = MarkdownParser::new();
        let markdown = "| Header |\n|--------|\n| Cell   |";
        let events: Vec<Event> = parser.parse(markdown).collect();
        assert!(!events.is_empty());
    }
}

