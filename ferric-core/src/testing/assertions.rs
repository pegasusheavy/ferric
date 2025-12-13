//! Test assertions for Ferric components.
//!
//! Provides fluent assertion helpers for testing components, services, and DOM elements.

use std::fmt;

/// Assertion result that can chain assertions.
#[derive(Debug)]
pub struct Assertion<T> {
    value: T,
    description: String,
}

impl<T> Assertion<T> {
    /// Create a new assertion.
    pub fn new(value: T) -> Self {
        Self {
            value,
            description: String::new(),
        }
    }

    /// Add a description for better error messages.
    pub fn described_as(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Get the underlying value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Map the value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Assertion<U> {
        Assertion {
            value: f(self.value),
            description: self.description,
        }
    }
}

impl<T: PartialEq + fmt::Debug> Assertion<T> {
    /// Assert equality.
    pub fn is_equal_to(&self, expected: &T) -> &Self {
        assert_eq!(
            &self.value, expected,
            "{}Expected {:?} but got {:?}",
            self.desc_prefix(),
            expected,
            self.value
        );
        self
    }

    /// Assert inequality.
    pub fn is_not_equal_to(&self, unexpected: &T) -> &Self {
        assert_ne!(
            &self.value, unexpected,
            "{}Expected value to not equal {:?}",
            self.desc_prefix(),
            unexpected
        );
        self
    }
}

impl<T: fmt::Debug> Assertion<T> {
    fn desc_prefix(&self) -> String {
        if self.description.is_empty() {
            String::new()
        } else {
            format!("[{}] ", self.description)
        }
    }
}

impl<T: PartialOrd + fmt::Debug> Assertion<T> {
    /// Assert greater than.
    pub fn is_greater_than(&self, other: &T) -> &Self {
        assert!(
            self.value > *other,
            "{}Expected {:?} to be greater than {:?}",
            self.desc_prefix(),
            self.value,
            other
        );
        self
    }

    /// Assert less than.
    pub fn is_less_than(&self, other: &T) -> &Self {
        assert!(
            self.value < *other,
            "{}Expected {:?} to be less than {:?}",
            self.desc_prefix(),
            self.value,
            other
        );
        self
    }

    /// Assert greater than or equal.
    pub fn is_at_least(&self, other: &T) -> &Self {
        assert!(
            self.value >= *other,
            "{}Expected {:?} to be at least {:?}",
            self.desc_prefix(),
            self.value,
            other
        );
        self
    }

    /// Assert less than or equal.
    pub fn is_at_most(&self, other: &T) -> &Self {
        assert!(
            self.value <= *other,
            "{}Expected {:?} to be at most {:?}",
            self.desc_prefix(),
            self.value,
            other
        );
        self
    }

    /// Assert value is in range.
    pub fn is_between(&self, min: &T, max: &T) -> &Self {
        assert!(
            self.value >= *min && self.value <= *max,
            "{}Expected {:?} to be between {:?} and {:?}",
            self.desc_prefix(),
            self.value,
            min,
            max
        );
        self
    }
}

impl Assertion<bool> {
    /// Assert value is true.
    pub fn is_true(&self) -> &Self {
        assert!(self.value, "{}Expected true but got false", self.desc_prefix());
        self
    }

    /// Assert value is false.
    pub fn is_false(&self) -> &Self {
        assert!(!self.value, "{}Expected false but got true", self.desc_prefix());
        self
    }
}

impl<T: fmt::Debug> Assertion<Option<T>> {
    /// Assert value is Some.
    pub fn is_some(&self) -> &Self {
        assert!(
            self.value.is_some(),
            "{}Expected Some but got None",
            self.desc_prefix()
        );
        self
    }

    /// Assert value is None.
    pub fn is_none(&self) -> &Self {
        assert!(
            self.value.is_none(),
            "{}Expected None but got {:?}",
            self.desc_prefix(),
            self.value
        );
        self
    }

    /// Unwrap and continue asserting on the inner value.
    pub fn unwrap(self) -> Assertion<T>
    where
        T: Clone,
    {
        self.is_some();
        Assertion {
            value: self.value.unwrap(),
            description: self.description,
        }
    }
}

impl<T: fmt::Debug, E: fmt::Debug> Assertion<Result<T, E>> {
    /// Assert value is Ok.
    pub fn is_ok(&self) -> &Self {
        assert!(
            self.value.is_ok(),
            "{}Expected Ok but got {:?}",
            self.desc_prefix(),
            self.value
        );
        self
    }

    /// Assert value is Err.
    pub fn is_err(&self) -> &Self {
        assert!(
            self.value.is_err(),
            "{}Expected Err but got {:?}",
            self.desc_prefix(),
            self.value
        );
        self
    }

    /// Unwrap Ok and continue asserting.
    pub fn unwrap_ok(self) -> Assertion<T>
    where
        T: Clone,
    {
        self.is_ok();
        Assertion {
            value: self.value.unwrap(),
            description: self.description,
        }
    }
}

impl Assertion<String> {
    /// Assert string is empty.
    pub fn is_empty(&self) -> &Self {
        assert!(
            self.value.is_empty(),
            "{}Expected empty string but got '{}'",
            self.desc_prefix(),
            self.value
        );
        self
    }

    /// Assert string is not empty.
    pub fn is_not_empty(&self) -> &Self {
        assert!(
            !self.value.is_empty(),
            "{}Expected non-empty string",
            self.desc_prefix()
        );
        self
    }

    /// Assert string contains substring.
    pub fn contains(&self, substring: &str) -> &Self {
        assert!(
            self.value.contains(substring),
            "{}Expected '{}' to contain '{}'",
            self.desc_prefix(),
            self.value,
            substring
        );
        self
    }

    /// Assert string starts with prefix.
    pub fn starts_with(&self, prefix: &str) -> &Self {
        assert!(
            self.value.starts_with(prefix),
            "{}Expected '{}' to start with '{}'",
            self.desc_prefix(),
            self.value,
            prefix
        );
        self
    }

    /// Assert string ends with suffix.
    pub fn ends_with(&self, suffix: &str) -> &Self {
        assert!(
            self.value.ends_with(suffix),
            "{}Expected '{}' to end with '{}'",
            self.desc_prefix(),
            self.value,
            suffix
        );
        self
    }

    /// Assert string matches pattern (simple glob-style matching).
    pub fn matches_pattern(&self, pattern: &str) -> &Self {
        // Simple pattern matching without regex dependency
        // Supports * as wildcard
        let matches = if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut remaining = self.value.as_str();
            let mut matched = true;

            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                if i == 0 {
                    // First part must be at start
                    if !remaining.starts_with(part) {
                        matched = false;
                        break;
                    }
                    remaining = &remaining[part.len()..];
                } else if i == parts.len() - 1 {
                    // Last part must be at end
                    if !remaining.ends_with(part) {
                        matched = false;
                        break;
                    }
                } else {
                    // Middle parts just need to exist
                    if let Some(pos) = remaining.find(part) {
                        remaining = &remaining[pos + part.len()..];
                    } else {
                        matched = false;
                        break;
                    }
                }
            }
            matched
        } else {
            self.value == pattern
        };

        assert!(
            matches,
            "{}Expected '{}' to match pattern '{}'",
            self.desc_prefix(),
            self.value,
            pattern
        );
        self
    }

    /// Assert string has length.
    pub fn has_length(&self, length: usize) -> &Self {
        assert_eq!(
            self.value.len(),
            length,
            "{}Expected string length {} but got {}",
            self.desc_prefix(),
            length,
            self.value.len()
        );
        self
    }
}

impl<T: fmt::Debug> Assertion<Vec<T>> {
    /// Assert vec is empty.
    pub fn is_empty(&self) -> &Self {
        assert!(
            self.value.is_empty(),
            "{}Expected empty vec but got {:?}",
            self.desc_prefix(),
            self.value
        );
        self
    }

    /// Assert vec is not empty.
    pub fn is_not_empty(&self) -> &Self {
        assert!(
            !self.value.is_empty(),
            "{}Expected non-empty vec",
            self.desc_prefix()
        );
        self
    }

    /// Assert vec has length.
    pub fn has_length(&self, length: usize) -> &Self {
        assert_eq!(
            self.value.len(),
            length,
            "{}Expected vec length {} but got {}",
            self.desc_prefix(),
            length,
            self.value.len()
        );
        self
    }

    /// Assert vec contains item.
    pub fn contains(&self, item: &T) -> &Self
    where
        T: PartialEq,
    {
        assert!(
            self.value.contains(item),
            "{}Expected vec to contain {:?}",
            self.desc_prefix(),
            item
        );
        self
    }

    /// Assert first element.
    pub fn first_is(&self, expected: &T) -> &Self
    where
        T: PartialEq,
    {
        let first = self.value.first();
        assert_eq!(
            first,
            Some(expected),
            "{}Expected first element to be {:?} but got {:?}",
            self.desc_prefix(),
            expected,
            first
        );
        self
    }

    /// Assert last element.
    pub fn last_is(&self, expected: &T) -> &Self
    where
        T: PartialEq,
    {
        let last = self.value.last();
        assert_eq!(
            last,
            Some(expected),
            "{}Expected last element to be {:?} but got {:?}",
            self.desc_prefix(),
            expected,
            last
        );
        self
    }
}

/// Start an assertion chain.
pub fn assert_that<T>(value: T) -> Assertion<T> {
    Assertion::new(value)
}

/// Assert a condition is true.
pub fn assert_true(condition: bool, message: &str) {
    assert!(condition, "{}", message);
}

/// Assert a condition is false.
pub fn assert_false(condition: bool, message: &str) {
    assert!(!condition, "{}", message);
}

/// Assert two values are equal.
pub fn assert_equals<T: PartialEq + fmt::Debug>(actual: T, expected: T) {
    assert_eq!(actual, expected);
}

/// Assert a function panics.
pub fn assert_panics<F: FnOnce() + std::panic::UnwindSafe>(f: F) {
    let result = std::panic::catch_unwind(f);
    assert!(result.is_err(), "Expected function to panic");
}

/// Assert a function does not panic.
pub fn assert_no_panic<F: FnOnce() + std::panic::UnwindSafe>(f: F) {
    let result = std::panic::catch_unwind(f);
    assert!(result.is_ok(), "Expected function to not panic");
}

/// Soft assertions that collect failures.
pub struct SoftAssertions {
    failures: Vec<String>,
}

impl SoftAssertions {
    /// Create new soft assertions.
    pub fn new() -> Self {
        Self {
            failures: Vec::new(),
        }
    }

    /// Add a check.
    pub fn check<T: PartialEq + fmt::Debug>(&mut self, actual: &T, expected: &T, message: &str) {
        if actual != expected {
            self.failures.push(format!(
                "{}: expected {:?} but got {:?}",
                message, expected, actual
            ));
        }
    }

    /// Add a boolean check.
    pub fn check_true(&mut self, condition: bool, message: &str) {
        if !condition {
            self.failures.push(format!("{}: expected true", message));
        }
    }

    /// Add a boolean check.
    pub fn check_false(&mut self, condition: bool, message: &str) {
        if condition {
            self.failures.push(format!("{}: expected false", message));
        }
    }

    /// Get failure count.
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Assert all checks passed.
    pub fn assert_all(&self) {
        if !self.failures.is_empty() {
            let message = format!(
                "Soft assertion failures:\n{}",
                self.failures
                    .iter()
                    .enumerate()
                    .map(|(i, f)| format!("  {}. {}", i + 1, f))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            panic!("{}", message);
        }
    }

    /// Check if all assertions passed.
    pub fn all_passed(&self) -> bool {
        self.failures.is_empty()
    }
}

impl Default for SoftAssertions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertion_equality() {
        assert_that(42).is_equal_to(&42);
    }

    #[test]
    fn test_assertion_comparison() {
        assert_that(10).is_greater_than(&5).is_less_than(&20);
    }

    #[test]
    fn test_assertion_bool() {
        assert_that(true).is_true();
        assert_that(false).is_false();
    }

    #[test]
    fn test_assertion_option() {
        assert_that(Some(42)).is_some();
        assert_that(None::<i32>).is_none();
    }

    #[test]
    fn test_assertion_string() {
        assert_that("hello world".to_string())
            .contains("world")
            .starts_with("hello")
            .ends_with("world");
    }

    #[test]
    fn test_assertion_vec() {
        assert_that(vec![1, 2, 3])
            .is_not_empty()
            .has_length(3)
            .contains(&2);
    }

    #[test]
    fn test_soft_assertions_all_pass() {
        let mut soft = SoftAssertions::new();
        soft.check(&1, &1, "check 1");
        soft.check_true(true, "check 2");
        soft.assert_all();
    }

    #[test]
    #[should_panic]
    fn test_soft_assertions_with_failures() {
        let mut soft = SoftAssertions::new();
        soft.check(&1, &2, "check 1");
        soft.assert_all();
    }

    #[test]
    fn test_assert_panics() {
        assert_panics(|| panic!("expected panic"));
    }
}
