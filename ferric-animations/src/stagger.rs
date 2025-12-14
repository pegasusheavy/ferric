//! Stagger animations
//!
//! Animate lists of elements with progressive delays.

use crate::timing::TimingFunction;
use serde::{Deserialize, Serialize};

/// Stagger configuration for list animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaggerConfig {
    /// Base timing for each item
    pub timing: TimingFunction,
    /// Delay between each item (in ms)
    pub stagger_delay: u32,
    /// Maximum number of items to stagger
    pub max_items: Option<usize>,
    /// Direction (forward or reverse)
    pub direction: StaggerDirection,
}

impl StaggerConfig {
    /// Create a new stagger configuration
    pub fn new(timing: TimingFunction, stagger_delay: u32) -> Self {
        Self {
            timing,
            stagger_delay,
            max_items: None,
            direction: StaggerDirection::Forward,
        }
    }

    /// Set maximum items
    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    /// Set direction
    pub fn with_direction(mut self, direction: StaggerDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Calculate delay for an item at given index
    pub fn delay_for_index(&self, index: usize, total: usize) -> u32 {
        let capped_index = if let Some(max) = self.max_items {
            index.min(max - 1)
        } else {
            index
        };

        let effective_index = match self.direction {
            StaggerDirection::Forward => capped_index,
            StaggerDirection::Reverse => {
                let max = self.max_items.unwrap_or(total);
                max.saturating_sub(capped_index + 1)
            }
        };

        self.timing.delay + (effective_index as u32 * self.stagger_delay)
    }

    /// Create timing for an item at given index
    pub fn timing_for_index(&self, index: usize, total: usize) -> TimingFunction {
        let mut timing = self.timing.clone();
        timing.delay = self.delay_for_index(index, total);
        timing
    }
}

/// Stagger direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaggerDirection {
    /// Items animate from first to last
    Forward,
    /// Items animate from last to first
    Reverse,
}

/// Helper function to create stagger configuration
///
/// # Example
///
/// ```rust,no_run
/// use ferric_animations::{stagger, TimingFunction, EasingFunction};
///
/// let config = stagger(
///     TimingFunction::new(300, 0, EasingFunction::Ease),
///     50 // 50ms delay between items
/// );
/// ```
pub fn stagger(timing: TimingFunction, delay: u32) -> StaggerConfig {
    StaggerConfig::new(timing, delay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::EasingFunction;

    fn sample_timing() -> TimingFunction {
        TimingFunction::new(300, 0, EasingFunction::Ease)
    }

    #[test]
    fn test_stagger_forward() {
        let config = StaggerConfig::new(sample_timing(), 50);

        assert_eq!(config.delay_for_index(0, 10), 0);
        assert_eq!(config.delay_for_index(1, 10), 50);
        assert_eq!(config.delay_for_index(2, 10), 100);
    }

    #[test]
    fn test_stagger_reverse() {
        let config = StaggerConfig::new(sample_timing(), 50)
            .with_direction(StaggerDirection::Reverse)
            .with_max_items(5);

        // With 5 items total, reverse means last item has 0 delay
        assert_eq!(config.delay_for_index(4, 5), 0);
        assert_eq!(config.delay_for_index(3, 5), 50);
        assert_eq!(config.delay_for_index(0, 5), 200);
    }

    #[test]
    fn test_stagger_max_items() {
        let config = StaggerConfig::new(sample_timing(), 50).with_max_items(3);

        // Only first 3 items get staggered
        assert_eq!(config.delay_for_index(0, 10), 0);
        assert_eq!(config.delay_for_index(2, 10), 100);
        assert_eq!(config.delay_for_index(5, 10), 100); // Capped at max
    }
}

