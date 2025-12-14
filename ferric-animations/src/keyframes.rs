//! Keyframe animations
//!
//! Support for multi-step keyframe animations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Keyframe animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    /// Offset (0.0 to 1.0)
    pub offset: f32,
    /// CSS styles at this keyframe
    pub styles: HashMap<String, String>,
}

impl Keyframe {
    /// Create a new keyframe
    pub fn new(offset: f32, styles: HashMap<String, String>) -> Self {
        Self { offset, styles }
    }

    /// Convert to CSS keyframe
    pub fn to_css(&self) -> String {
        let styles: Vec<String> = self
            .styles
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();

        format!("{}% {{ {} }}", (self.offset * 100.0) as u8, styles.join("; "))
    }
}

/// Keyframe sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeSequence {
    /// Name of the animation
    pub name: String,
    /// Keyframes
    pub keyframes: Vec<Keyframe>,
}

impl KeyframeSequence {
    /// Create a new keyframe sequence
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            keyframes: Vec::new(),
        }
    }

    /// Add a keyframe
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        // Sort by offset
        self.keyframes.sort_by(|a, b| {
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Convert to CSS @keyframes rule
    pub fn to_css(&self) -> String {
        let frames: Vec<String> = self.keyframes.iter().map(|k| k.to_css()).collect();

        format!("@keyframes {} {{\n  {}\n}}", self.name, frames.join("\n  "))
    }
}

/// Helper function to create keyframes
///
/// # Example
///
/// ```rust,no_run
/// use ferric_animations::keyframes;
///
/// let bounce = keyframes("bounce", vec![
///     (0.0, vec![("transform", "translateY(0)")]),
///     (0.5, vec![("transform", "translateY(-20px)")]),
///     (1.0, vec![("transform", "translateY(0)")]),
/// ]);
/// ```
pub fn keyframes(
    name: impl Into<String>,
    frames: Vec<(f32, Vec<(&str, &str)>)>,
) -> KeyframeSequence {
    let mut sequence = KeyframeSequence::new(name);

    for (offset, styles) in frames {
        let styles = styles
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        sequence.add_keyframe(Keyframe::new(offset, styles));
    }

    sequence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_creation() {
        let k = Keyframe::new(
            0.5,
            vec![("opacity".to_string(), "0.5".to_string())]
                .into_iter()
                .collect(),
        );
        assert_eq!(k.offset, 0.5);
    }

    #[test]
    fn test_keyframe_sequence() {
        let mut seq = KeyframeSequence::new("test");
        seq.add_keyframe(Keyframe::new(
            0.0,
            vec![("opacity".to_string(), "0".to_string())]
                .into_iter()
                .collect(),
        ));
        seq.add_keyframe(Keyframe::new(
            1.0,
            vec![("opacity".to_string(), "1".to_string())]
                .into_iter()
                .collect(),
        ));

        assert_eq!(seq.keyframes.len(), 2);
        assert!(seq.to_css().contains("@keyframes test"));
    }

    #[test]
    fn test_keyframes_helper() {
        let bounce = keyframes(
            "bounce",
            vec![
                (0.0, vec![("transform", "translateY(0)")]),
                (0.5, vec![("transform", "translateY(-20px)")]),
                (1.0, vec![("transform", "translateY(0)")]),
            ],
        );

        assert_eq!(bounce.keyframes.len(), 3);
        assert_eq!(bounce.name, "bounce");
    }
}

