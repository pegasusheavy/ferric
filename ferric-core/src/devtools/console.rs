//! Browser console integration for enhanced debugging.
//!
//! Provides utilities for interacting with the browser console,
//! including formatted output, grouping, and performance marks.

use std::cell::RefCell;

/// Console logging utilities.
pub struct Console;

impl Console {
    /// Log a message with styling.
    #[cfg(target_arch = "wasm32")]
    pub fn log(message: &str) {
        web_sys::console::log_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn log(message: &str) {
        println!("{}", message);
    }

    /// Log a warning.
    #[cfg(target_arch = "wasm32")]
    pub fn warn(message: &str) {
        web_sys::console::warn_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn warn(message: &str) {
        eprintln!("WARN: {}", message);
    }

    /// Log an error.
    #[cfg(target_arch = "wasm32")]
    pub fn error(message: &str) {
        web_sys::console::error_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn error(message: &str) {
        eprintln!("ERROR: {}", message);
    }

    /// Log debug info (only in debug mode).
    #[cfg(target_arch = "wasm32")]
    pub fn debug(message: &str) {
        web_sys::console::debug_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn debug(message: &str) {
        println!("DEBUG: {}", message);
    }

    /// Log with info level.
    #[cfg(target_arch = "wasm32")]
    pub fn info(message: &str) {
        web_sys::console::info_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn info(message: &str) {
        println!("INFO: {}", message);
    }

    /// Start a console group.
    #[cfg(target_arch = "wasm32")]
    pub fn group(label: &str) {
        web_sys::console::group_1(&label.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn group(label: &str) {
        println!("=== {} ===", label);
    }

    /// Start a collapsed console group.
    #[cfg(target_arch = "wasm32")]
    pub fn group_collapsed(label: &str) {
        web_sys::console::group_collapsed_1(&label.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn group_collapsed(label: &str) {
        println!(">>> {} (collapsed)", label);
    }

    /// End a console group.
    #[cfg(target_arch = "wasm32")]
    pub fn group_end() {
        web_sys::console::group_end();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn group_end() {
        println!("==========");
    }

    /// Start a timer.
    #[cfg(target_arch = "wasm32")]
    pub fn time(label: &str) {
        web_sys::console::time_with_label(label);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn time(label: &str) {
        println!("TIMER START: {}", label);
    }

    /// End a timer and log the duration.
    #[cfg(target_arch = "wasm32")]
    pub fn time_end(label: &str) {
        web_sys::console::time_end_with_label(label);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn time_end(label: &str) {
        println!("TIMER END: {}", label);
    }

    /// Log a timer checkpoint.
    #[cfg(target_arch = "wasm32")]
    pub fn time_log(label: &str) {
        web_sys::console::time_log_with_label(label);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn time_log(label: &str) {
        println!("TIMER LOG: {}", label);
    }

    /// Log a table.
    #[cfg(target_arch = "wasm32")]
    pub fn table(data: &wasm_bindgen::JsValue) {
        web_sys::console::table_1(data);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn table<T: std::fmt::Debug>(data: &T) {
        println!("TABLE: {:?}", data);
    }

    /// Clear the console.
    #[cfg(target_arch = "wasm32")]
    pub fn clear() {
        web_sys::console::clear();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear() {
        // Clear screen on native
        print!("\x1B[2J\x1B[1;1H");
    }

    /// Count occurrences.
    #[cfg(target_arch = "wasm32")]
    pub fn count(label: &str) {
        web_sys::console::count_with_label(label);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn count(label: &str) {
        COUNTS.with(|c| {
            let mut counts = c.borrow_mut();
            let count = counts.entry(label.to_string()).or_insert(0);
            *count += 1;
            println!("{}: {}", label, count);
        });
    }

    /// Reset count.
    #[cfg(target_arch = "wasm32")]
    pub fn count_reset(label: &str) {
        web_sys::console::count_reset_with_label(label);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn count_reset(label: &str) {
        COUNTS.with(|c| {
            c.borrow_mut().remove(label);
        });
        println!("{}: count reset", label);
    }

    /// Assert a condition.
    #[cfg(target_arch = "wasm32")]
    pub fn assert(condition: bool, message: &str) {
        web_sys::console::assert_with_condition_and_data_1(condition, &message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn assert(condition: bool, message: &str) {
        if !condition {
            eprintln!("ASSERTION FAILED: {}", message);
        }
    }

    /// Add a performance mark.
    #[cfg(target_arch = "wasm32")]
    pub fn mark(name: &str) {
        if let Some(window) = web_sys::window() {
            if let Ok(perf) = window.performance() {
                if let Some(p) = perf {
                    p.mark(name).ok();
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn mark(name: &str) {
        println!("MARK: {}", name);
    }

    /// Measure between two marks.
    #[cfg(target_arch = "wasm32")]
    pub fn measure(name: &str, start: &str, end: &str) {
        if let Some(window) = web_sys::window() {
            if let Ok(perf) = window.performance() {
                if let Some(p) = perf {
                    p.measure_with_start_mark_and_end_mark(name, start, end).ok();
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn measure(name: &str, start: &str, end: &str) {
        println!("MEASURE: {} ({} -> {})", name, start, end);
    }
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static COUNTS: RefCell<std::collections::HashMap<String, u32>> = RefCell::new(std::collections::HashMap::new());
}

/// Formatted console output with styles.
pub struct StyledLog {
    parts: Vec<(String, String)>,
}

impl StyledLog {
    /// Create a new styled log.
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Add text with a style.
    pub fn add(mut self, text: &str, style: &str) -> Self {
        self.parts.push((text.to_string(), style.to_string()));
        self
    }

    /// Add text with bold style.
    pub fn bold(self, text: &str) -> Self {
        self.add(text, "font-weight: bold")
    }

    /// Add text with color.
    pub fn color(self, text: &str, color: &str) -> Self {
        self.add(text, &format!("color: {}", color))
    }

    /// Add text with background color.
    pub fn background(self, text: &str, color: &str) -> Self {
        self.add(text, &format!("background-color: {}; padding: 2px 4px; border-radius: 2px", color))
    }

    /// Add text styled as success (green).
    pub fn success(self, text: &str) -> Self {
        self.add(text, "color: #4caf50; font-weight: bold")
    }

    /// Add text styled as error (red).
    pub fn error(self, text: &str) -> Self {
        self.add(text, "color: #f44336; font-weight: bold")
    }

    /// Add text styled as warning (orange).
    pub fn warning(self, text: &str) -> Self {
        self.add(text, "color: #ff9800; font-weight: bold")
    }

    /// Add text styled as info (blue).
    pub fn info(self, text: &str) -> Self {
        self.add(text, "color: #2196f3; font-weight: bold")
    }

    /// Add plain text.
    pub fn plain(self, text: &str) -> Self {
        self.add(text, "")
    }

    /// Log the styled message.
    #[cfg(target_arch = "wasm32")]
    pub fn log(self) {
        let format_string: String = self.parts.iter().map(|_| "%c%s").collect();
        let args: Vec<wasm_bindgen::JsValue> = self
            .parts
            .iter()
            .flat_map(|(text, style)| {
                vec![
                    wasm_bindgen::JsValue::from_str(style),
                    wasm_bindgen::JsValue::from_str(text),
                ]
            })
            .collect();

        // Use console.log with format string
        let array = js_sys::Array::new();
        array.push(&format_string.into());
        for arg in args {
            array.push(&arg);
        }
        web_sys::console::log(&array);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn log(self) {
        let message: String = self.parts.iter().map(|(text, _)| text.as_str()).collect();
        println!("{}", message);
    }
}

impl Default for StyledLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Console table builder.
pub struct TableBuilder {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl TableBuilder {
    /// Create a new table builder.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Set the headers.
    pub fn headers(mut self, headers: &[&str]) -> Self {
        self.headers = headers.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a row.
    pub fn row(mut self, values: &[&str]) -> Self {
        self.rows.push(values.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Print the table.
    pub fn print(self) {
        if self.headers.is_empty() && self.rows.is_empty() {
            return;
        }

        // Calculate column widths
        let mut widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();
        if widths.is_empty() && !self.rows.is_empty() {
            widths = vec![0; self.rows[0].len()];
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Print separator
        let separator: String = widths.iter().map(|w| "-".repeat(*w + 2)).collect::<Vec<_>>().join("+");
        let separator = format!("+{}+", separator);

        // Print headers
        if !self.headers.is_empty() {
            Console::log(&separator);
            let header_row: String = self
                .headers
                .iter()
                .enumerate()
                .map(|(i, h)| format!(" {:width$} ", h, width = widths.get(i).copied().unwrap_or(0)))
                .collect::<Vec<_>>()
                .join("|");
            Console::log(&format!("|{}|", header_row));
            Console::log(&separator);
        }

        // Print rows
        for row in &self.rows {
            let row_str: String = row
                .iter()
                .enumerate()
                .map(|(i, c)| format!(" {:width$} ", c, width = widths.get(i).copied().unwrap_or(0)))
                .collect::<Vec<_>>()
                .join("|");
            Console::log(&format!("|{}|", row_str));
        }

        if !self.rows.is_empty() {
            Console::log(&separator);
        }
    }
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_methods() {
        // Just verify methods don't panic
        Console::log("test log");
        Console::warn("test warn");
        Console::error("test error");
        Console::debug("test debug");
        Console::info("test info");
    }

    #[test]
    fn test_styled_log() {
        let log = StyledLog::new()
            .bold("Bold")
            .plain(" ")
            .success("Success")
            .plain(" ")
            .error("Error");

        // Shouldn't panic
        log.log();
    }

    #[test]
    fn test_table_builder() {
        let table = TableBuilder::new()
            .headers(&["Name", "Value"])
            .row(&["foo", "1"])
            .row(&["bar", "2"]);

        // Shouldn't panic
        table.print();
    }

    #[test]
    fn test_console_groups() {
        Console::group("Test Group");
        Console::log("Inside group");
        Console::group_end();
    }

    #[test]
    fn test_console_count() {
        Console::count("test");
        Console::count("test");
        Console::count_reset("test");
    }
}
