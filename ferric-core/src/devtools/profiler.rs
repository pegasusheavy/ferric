//! Performance profiler for change detection and component rendering.
//!
//! Tracks timing information for:
//! - Change detection cycles
//! - Component renders
//! - Effect executions
//! - Signal updates

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance profiler for the framework.
pub struct Profiler {
    recording: RefCell<bool>,
    records: RefCell<Vec<ProfileRecord>>,
    current_frame: RefCell<Option<FrameProfile>>,
    component_stats: RefCell<HashMap<String, ComponentStats>>,
    settings: ProfilerSettings,
}

impl Profiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self {
            recording: RefCell::new(false),
            records: RefCell::new(Vec::new()),
            current_frame: RefCell::new(None),
            component_stats: RefCell::new(HashMap::new()),
            settings: ProfilerSettings::default(),
        }
    }

    /// Create a profiler with custom settings.
    pub fn with_settings(settings: ProfilerSettings) -> Self {
        Self {
            recording: RefCell::new(false),
            records: RefCell::new(Vec::new()),
            current_frame: RefCell::new(None),
            component_stats: RefCell::new(HashMap::new()),
            settings,
        }
    }

    /// Start recording.
    pub fn start_recording(&self) {
        *self.recording.borrow_mut() = true;
        self.records.borrow_mut().clear();
    }

    /// Stop recording and return a report.
    pub fn stop_recording(&self) -> ProfileReport {
        *self.recording.borrow_mut() = false;
        ProfileReport::from_records(self.records.borrow().clone())
    }

    /// Check if recording.
    pub fn is_recording(&self) -> bool {
        *self.recording.borrow()
    }

    /// Start a new frame.
    pub fn start_frame(&self) {
        if !*self.recording.borrow() {
            return;
        }
        *self.current_frame.borrow_mut() = Some(FrameProfile::new());
    }

    /// End the current frame.
    pub fn end_frame(&self) {
        if !*self.recording.borrow() {
            return;
        }

        if let Some(frame) = self.current_frame.borrow_mut().take() {
            self.records.borrow_mut().push(ProfileRecord::Frame(frame));
        }
    }

    /// Record a change detection cycle.
    pub fn record_change_detection(&self, component_id: &str, duration: Duration, dirty: bool) {
        if !*self.recording.borrow() {
            return;
        }

        let record = ChangeDetectionRecord {
            component_id: component_id.to_string(),
            duration,
            dirty,
            timestamp: Instant::now(),
        };

        // Add to current frame if active
        if let Some(ref mut frame) = *self.current_frame.borrow_mut() {
            frame.change_detections.push(record.clone());
        }

        self.records.borrow_mut().push(ProfileRecord::ChangeDetection(record));

        // Update component stats
        self.update_component_stats(component_id, duration, ProfileEventType::ChangeDetection);
    }

    /// Record a component render.
    pub fn record_render(&self, component_id: &str, duration: Duration) {
        if !*self.recording.borrow() {
            return;
        }

        let record = RenderRecord {
            component_id: component_id.to_string(),
            duration,
            timestamp: Instant::now(),
        };

        if let Some(ref mut frame) = *self.current_frame.borrow_mut() {
            frame.renders.push(record.clone());
        }

        self.records.borrow_mut().push(ProfileRecord::Render(record));

        self.update_component_stats(component_id, duration, ProfileEventType::Render);
    }

    /// Record an effect execution.
    pub fn record_effect(&self, effect_id: &str, duration: Duration) {
        if !*self.recording.borrow() {
            return;
        }

        let record = EffectRecord {
            effect_id: effect_id.to_string(),
            duration,
            timestamp: Instant::now(),
        };

        if let Some(ref mut frame) = *self.current_frame.borrow_mut() {
            frame.effects.push(record.clone());
        }

        self.records.borrow_mut().push(ProfileRecord::Effect(record));
    }

    /// Record a signal update.
    pub fn record_signal_update(&self, signal_id: &str, duration: Duration) {
        if !*self.recording.borrow() {
            return;
        }

        let record = SignalRecord {
            signal_id: signal_id.to_string(),
            duration,
            timestamp: Instant::now(),
        };

        if let Some(ref mut frame) = *self.current_frame.borrow_mut() {
            frame.signal_updates.push(record.clone());
        }

        self.records.borrow_mut().push(ProfileRecord::Signal(record));
    }

    /// Update component statistics.
    fn update_component_stats(&self, component_id: &str, duration: Duration, event_type: ProfileEventType) {
        let mut stats = self.component_stats.borrow_mut();
        let entry = stats.entry(component_id.to_string()).or_insert_with(|| ComponentStats {
            component_id: component_id.to_string(),
            render_count: 0,
            cd_count: 0,
            total_render_time: Duration::ZERO,
            total_cd_time: Duration::ZERO,
            max_render_time: Duration::ZERO,
            max_cd_time: Duration::ZERO,
        });

        match event_type {
            ProfileEventType::Render => {
                entry.render_count += 1;
                entry.total_render_time += duration;
                if duration > entry.max_render_time {
                    entry.max_render_time = duration;
                }
            }
            ProfileEventType::ChangeDetection => {
                entry.cd_count += 1;
                entry.total_cd_time += duration;
                if duration > entry.max_cd_time {
                    entry.max_cd_time = duration;
                }
            }
            _ => {}
        }
    }

    /// Get component statistics.
    pub fn get_component_stats(&self, component_id: &str) -> Option<ComponentStats> {
        self.component_stats.borrow().get(component_id).cloned()
    }

    /// Get all component statistics.
    pub fn get_all_stats(&self) -> Vec<ComponentStats> {
        self.component_stats.borrow().values().cloned().collect()
    }

    /// Clear all records and stats.
    pub fn clear(&self) {
        self.records.borrow_mut().clear();
        self.component_stats.borrow_mut().clear();
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiler settings.
#[derive(Debug, Clone)]
pub struct ProfilerSettings {
    /// Maximum number of records to keep.
    pub max_records: usize,
    /// Whether to track signal updates.
    pub track_signals: bool,
    /// Whether to track effects.
    pub track_effects: bool,
    /// Minimum duration to record (filter out fast operations).
    pub min_duration: Duration,
}

impl Default for ProfilerSettings {
    fn default() -> Self {
        Self {
            max_records: 10000,
            track_signals: true,
            track_effects: true,
            min_duration: Duration::ZERO,
        }
    }
}

/// Profile record types.
#[derive(Debug, Clone)]
pub enum ProfileRecord {
    /// Full frame profile.
    Frame(FrameProfile),
    /// Change detection event.
    ChangeDetection(ChangeDetectionRecord),
    /// Render event.
    Render(RenderRecord),
    /// Effect execution.
    Effect(EffectRecord),
    /// Signal update.
    Signal(SignalRecord),
}

/// Profile event types.
#[derive(Debug, Clone, Copy)]
pub enum ProfileEventType {
    ChangeDetection,
    Render,
    Effect,
    Signal,
}

/// Profile for a single frame.
#[derive(Debug, Clone)]
pub struct FrameProfile {
    pub start: Instant,
    pub end: Option<Instant>,
    pub change_detections: Vec<ChangeDetectionRecord>,
    pub renders: Vec<RenderRecord>,
    pub effects: Vec<EffectRecord>,
    pub signal_updates: Vec<SignalRecord>,
}

impl FrameProfile {
    /// Create a new frame profile.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            end: None,
            change_detections: Vec::new(),
            renders: Vec::new(),
            effects: Vec::new(),
            signal_updates: Vec::new(),
        }
    }

    /// Get frame duration.
    pub fn duration(&self) -> Duration {
        self.end.map(|e| e - self.start).unwrap_or(self.start.elapsed())
    }

    /// Get total change detection time.
    pub fn total_cd_time(&self) -> Duration {
        self.change_detections.iter().map(|r| r.duration).sum()
    }

    /// Get total render time.
    pub fn total_render_time(&self) -> Duration {
        self.renders.iter().map(|r| r.duration).sum()
    }
}

impl Default for FrameProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Change detection record.
#[derive(Debug, Clone)]
pub struct ChangeDetectionRecord {
    pub component_id: String,
    pub duration: Duration,
    pub dirty: bool,
    pub timestamp: Instant,
}

/// Render record.
#[derive(Debug, Clone)]
pub struct RenderRecord {
    pub component_id: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// Effect execution record.
#[derive(Debug, Clone)]
pub struct EffectRecord {
    pub effect_id: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// Signal update record.
#[derive(Debug, Clone)]
pub struct SignalRecord {
    pub signal_id: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// Component statistics.
#[derive(Debug, Clone)]
pub struct ComponentStats {
    pub component_id: String,
    pub render_count: u64,
    pub cd_count: u64,
    pub total_render_time: Duration,
    pub total_cd_time: Duration,
    pub max_render_time: Duration,
    pub max_cd_time: Duration,
}

impl ComponentStats {
    /// Get average render time.
    pub fn avg_render_time(&self) -> Duration {
        if self.render_count > 0 {
            self.total_render_time / self.render_count as u32
        } else {
            Duration::ZERO
        }
    }

    /// Get average change detection time.
    pub fn avg_cd_time(&self) -> Duration {
        if self.cd_count > 0 {
            self.total_cd_time / self.cd_count as u32
        } else {
            Duration::ZERO
        }
    }
}

/// Profile report generated after recording.
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// Recording duration.
    pub duration: Duration,
    /// Total records.
    pub record_count: usize,
    /// Frame count.
    pub frame_count: usize,
    /// Average frame time.
    pub avg_frame_time: Duration,
    /// Max frame time.
    pub max_frame_time: Duration,
    /// Total change detection count.
    pub total_cd_count: usize,
    /// Average change detection time.
    pub avg_cd_time: Duration,
    /// Total render count.
    pub total_render_count: usize,
    /// Average render time.
    pub avg_render_time: Duration,
    /// Slowest components.
    pub slowest_components: Vec<(String, Duration)>,
    /// Most frequently rendered components.
    pub most_rendered: Vec<(String, u64)>,
    /// Raw records (for detailed analysis).
    pub records: Vec<ProfileRecord>,
}

impl ProfileReport {
    /// Create a report from recorded data.
    pub fn from_records(records: Vec<ProfileRecord>) -> Self {
        let mut frames = Vec::new();
        let mut cd_records = Vec::new();
        let mut render_records = Vec::new();
        let mut component_times: HashMap<String, Duration> = HashMap::new();
        let mut component_counts: HashMap<String, u64> = HashMap::new();

        for record in &records {
            match record {
                ProfileRecord::Frame(f) => {
                    frames.push(f.clone());
                }
                ProfileRecord::ChangeDetection(r) => {
                    cd_records.push(r.clone());
                }
                ProfileRecord::Render(r) => {
                    render_records.push(r.clone());
                    *component_times.entry(r.component_id.clone()).or_default() += r.duration;
                    *component_counts.entry(r.component_id.clone()).or_default() += 1;
                }
                _ => {}
            }
        }

        let frame_times: Vec<Duration> = frames.iter().map(|f| f.duration()).collect();
        let avg_frame_time = if !frame_times.is_empty() {
            frame_times.iter().sum::<Duration>() / frame_times.len() as u32
        } else {
            Duration::ZERO
        };
        let max_frame_time = frame_times.iter().max().copied().unwrap_or(Duration::ZERO);

        let cd_times: Vec<Duration> = cd_records.iter().map(|r| r.duration).collect();
        let avg_cd_time = if !cd_times.is_empty() {
            cd_times.iter().sum::<Duration>() / cd_times.len() as u32
        } else {
            Duration::ZERO
        };

        let render_times: Vec<Duration> = render_records.iter().map(|r| r.duration).collect();
        let avg_render_time = if !render_times.is_empty() {
            render_times.iter().sum::<Duration>() / render_times.len() as u32
        } else {
            Duration::ZERO
        };

        let mut slowest: Vec<_> = component_times.into_iter().collect();
        slowest.sort_by(|a, b| b.1.cmp(&a.1));
        slowest.truncate(10);

        let mut most_rendered: Vec<_> = component_counts.into_iter().collect();
        most_rendered.sort_by(|a, b| b.1.cmp(&a.1));
        most_rendered.truncate(10);

        Self {
            duration: Duration::ZERO, // Would need start/end timestamps
            record_count: records.len(),
            frame_count: frames.len(),
            avg_frame_time,
            max_frame_time,
            total_cd_count: cd_records.len(),
            avg_cd_time,
            total_render_count: render_records.len(),
            avg_render_time,
            slowest_components: slowest,
            most_rendered,
            records,
        }
    }

    /// Print a summary to console.
    pub fn print_summary(&self) {
        println!("=== Ferric Performance Report ===");
        println!("Records: {}", self.record_count);
        println!("Frames: {}", self.frame_count);
        println!("Avg Frame Time: {:?}", self.avg_frame_time);
        println!("Max Frame Time: {:?}", self.max_frame_time);
        println!("Change Detections: {}", self.total_cd_count);
        println!("Avg CD Time: {:?}", self.avg_cd_time);
        println!("Renders: {}", self.total_render_count);
        println!("Avg Render Time: {:?}", self.avg_render_time);
        println!("\nSlowest Components:");
        for (name, time) in &self.slowest_components {
            println!("  {}: {:?}", name, time);
        }
        println!("\nMost Rendered:");
        for (name, count) in &self.most_rendered {
            println!("  {}: {} times", name, count);
        }
    }

    /// Convert to JSON.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"frames":{},"avgFrameTime":"{}ms","maxFrameTime":"{}ms","changeDetections":{},"renders":{},"avgRenderTime":"{}ms"}}"#,
            self.frame_count,
            self.avg_frame_time.as_millis(),
            self.max_frame_time.as_millis(),
            self.total_cd_count,
            self.total_render_count,
            self.avg_render_time.as_millis(),
        )
    }
}

/// Profile guard that automatically records timing.
pub struct ProfileGuard<'a> {
    profiler: &'a Profiler,
    component_id: String,
    event_type: ProfileEventType,
    start: Instant,
}

impl<'a> ProfileGuard<'a> {
    /// Create a new profile guard.
    pub fn new(profiler: &'a Profiler, component_id: &str, event_type: ProfileEventType) -> Self {
        Self {
            profiler,
            component_id: component_id.to_string(),
            event_type,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for ProfileGuard<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        match self.event_type {
            ProfileEventType::Render => {
                self.profiler.record_render(&self.component_id, duration);
            }
            ProfileEventType::ChangeDetection => {
                self.profiler.record_change_detection(&self.component_id, duration, true);
            }
            _ => {}
        }
    }
}

/// Macro for profiling a block of code.
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $component:expr, $type:expr, $block:expr) => {{
        let _guard = $crate::devtools::ProfileGuard::new($profiler, $component, $type);
        $block
    }};
}

/// Global profiler instance.
thread_local! {
    static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::new());
}

/// Get the global profiler.
pub fn profiler() -> Profiler {
    PROFILER.with(|_p| {
        // Return a reference-like by creating new instance
        // In real code, we'd use Rc or similar
        Profiler::new()
    })
}

/// Start global profiling.
pub fn start_profiling() {
    PROFILER.with(|p| p.borrow().start_recording());
}

/// Stop global profiling and get report.
pub fn stop_profiling() -> ProfileReport {
    PROFILER.with(|p| p.borrow().stop_recording())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_recording() {
        let profiler = Profiler::new();

        profiler.start_recording();
        assert!(profiler.is_recording());

        profiler.record_render("comp-1", Duration::from_millis(5));
        profiler.record_render("comp-2", Duration::from_millis(10));
        profiler.record_change_detection("comp-1", Duration::from_millis(2), true);

        let report = profiler.stop_recording();
        assert!(!profiler.is_recording());

        assert_eq!(report.total_render_count, 2);
        assert_eq!(report.total_cd_count, 1);
    }

    #[test]
    fn test_component_stats() {
        let profiler = Profiler::new();
        profiler.start_recording();

        profiler.record_render("comp-1", Duration::from_millis(5));
        profiler.record_render("comp-1", Duration::from_millis(10));
        profiler.record_render("comp-1", Duration::from_millis(15));

        let stats = profiler.get_component_stats("comp-1").unwrap();
        assert_eq!(stats.render_count, 3);
        assert_eq!(stats.total_render_time, Duration::from_millis(30));
        assert_eq!(stats.max_render_time, Duration::from_millis(15));
    }

    #[test]
    fn test_frame_profile() {
        let profiler = Profiler::new();
        profiler.start_recording();

        profiler.start_frame();
        profiler.record_render("comp-1", Duration::from_millis(5));
        profiler.record_change_detection("comp-1", Duration::from_millis(2), true);
        profiler.end_frame();

        let report = profiler.stop_recording();
        assert_eq!(report.frame_count, 1);
    }

    #[test]
    fn test_profile_report() {
        let records = vec![
            ProfileRecord::Render(RenderRecord {
                component_id: "comp-1".to_string(),
                duration: Duration::from_millis(10),
                timestamp: Instant::now(),
            }),
            ProfileRecord::Render(RenderRecord {
                component_id: "comp-2".to_string(),
                duration: Duration::from_millis(20),
                timestamp: Instant::now(),
            }),
        ];

        let report = ProfileReport::from_records(records);
        assert_eq!(report.total_render_count, 2);
        assert!(!report.slowest_components.is_empty());
    }
}


