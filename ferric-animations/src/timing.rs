//! Animation timing functions
//!
//! Defines easing functions and timing configurations.

use serde::{Deserialize, Serialize};

/// Timing function for animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingFunction {
    /// Duration in milliseconds
    pub duration: u32,
    /// Delay in milliseconds
    pub delay: u32,
    /// Easing function
    pub easing: EasingFunction,
}

impl TimingFunction {
    /// Create a new timing function
    pub fn new(duration: u32, delay: u32, easing: EasingFunction) -> Self {
        Self {
            duration,
            delay,
            easing,
        }
    }

    /// Parse timing from string (e.g., "300ms ease-in")
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty timing string".to_string());
        }

        let duration = Self::parse_duration(parts[0])?;
        let delay = if parts.len() > 1 && parts[1].ends_with("ms") {
            Self::parse_duration(parts[1])?
        } else {
            0
        };

        let easing = if parts.len() > 1 {
            let easing_str = if delay > 0 && parts.len() > 2 {
                parts[2]
            } else if delay == 0 {
                parts[1]
            } else {
                "ease"
            };
            EasingFunction::parse(easing_str)?
        } else {
            EasingFunction::Ease
        };

        Ok(Self::new(duration, delay, easing))
    }

    fn parse_duration(s: &str) -> Result<u32, String> {
        if let Some(ms_str) = s.strip_suffix("ms") {
            ms_str
                .parse()
                .map_err(|_| format!("Invalid duration: {}", s))
        } else if let Some(s_str) = s.strip_suffix('s') {
            s_str
                .parse::<f32>()
                .map(|s| (s * 1000.0) as u32)
                .map_err(|_| format!("Invalid duration: {}", s))
        } else {
            Err(format!("Duration must end with 'ms' or 's': {}", s))
        }
    }

    /// Convert to CSS animation string
    pub fn to_css(&self) -> String {
        format!(
            "{}ms {} {}ms",
            self.duration,
            self.easing.to_css(),
            self.delay
        )
    }
}

/// Easing functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(i32, i32, i32, i32), // Stored as integers (x1*100, y1*100, x2*100, y2*100)
}

impl EasingFunction {
    /// Parse easing function from string
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "linear" => Ok(Self::Linear),
            "ease" => Ok(Self::Ease),
            "ease-in" => Ok(Self::EaseIn),
            "ease-out" => Ok(Self::EaseOut),
            "ease-in-out" => Ok(Self::EaseInOut),
            s if s.starts_with("cubic-bezier(") => {
                // Parse cubic-bezier(0.42, 0, 0.58, 1)
                let inner = s
                    .strip_prefix("cubic-bezier(")
                    .and_then(|s| s.strip_suffix(')'))
                    .ok_or_else(|| "Invalid cubic-bezier format".to_string())?;

                let parts: Vec<f32> = inner
                    .split(',')
                    .map(|p| p.trim().parse())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| "Invalid cubic-bezier values".to_string())?;

                if parts.len() != 4 {
                    return Err("cubic-bezier requires 4 values".to_string());
                }

                Ok(Self::CubicBezier(
                    (parts[0] * 100.0) as i32,
                    (parts[1] * 100.0) as i32,
                    (parts[2] * 100.0) as i32,
                    (parts[3] * 100.0) as i32,
                ))
            }
            _ => Err(format!("Unknown easing function: {}", s)),
        }
    }

    /// Convert to CSS easing string
    pub fn to_css(&self) -> String {
        match self {
            Self::Linear => "linear".to_string(),
            Self::Ease => "ease".to_string(),
            Self::EaseIn => "ease-in".to_string(),
            Self::EaseOut => "ease-out".to_string(),
            Self::EaseInOut => "ease-in-out".to_string(),
            Self::CubicBezier(x1, y1, x2, y2) => {
                format!(
                    "cubic-bezier({}, {}, {}, {})",
                    *x1 as f32 / 100.0,
                    *y1 as f32 / 100.0,
                    *x2 as f32 / 100.0,
                    *y2 as f32 / 100.0
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timing() {
        let t = TimingFunction::parse("300ms ease-in").unwrap();
        assert_eq!(t.duration, 300);
        assert_eq!(t.delay, 0);
        assert_eq!(t.easing, EasingFunction::EaseIn);
    }

    #[test]
    fn test_parse_timing_with_delay() {
        let t = TimingFunction::parse("300ms 100ms ease-out").unwrap();
        assert_eq!(t.duration, 300);
        assert_eq!(t.delay, 100);
        assert_eq!(t.easing, EasingFunction::EaseOut);
    }

    #[test]
    fn test_parse_cubic_bezier() {
        let e = EasingFunction::parse("cubic-bezier(0.42, 0, 0.58, 1)").unwrap();
        assert!(matches!(e, EasingFunction::CubicBezier(..)));
    }

    #[test]
    fn test_to_css() {
        let t = TimingFunction::new(300, 100, EasingFunction::Ease);
        assert_eq!(t.to_css(), "300ms ease 100ms");
    }
}

