//! State debugging and inspection utilities.
//!
//! Provides tools for tracking and visualizing application state,
//! including signal history, state snapshots, and state diffing.

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

/// State change event for history tracking.
#[derive(Debug, Clone)]
pub struct StateChange {
    /// Unique ID for the signal/state.
    pub id: String,
    /// Name/label for the state.
    pub name: String,
    /// Old value (JSON serialized).
    pub old_value: String,
    /// New value (JSON serialized).
    pub new_value: String,
    /// Timestamp of the change.
    pub timestamp: Instant,
    /// Source of the change (e.g., component name, function).
    pub source: Option<String>,
    /// Stack trace (in debug mode).
    pub stack_trace: Option<String>,
}

impl StateChange {
    /// Create a new state change.
    pub fn new(id: &str, name: &str, old: &str, new: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            old_value: old.to_string(),
            new_value: new.to_string(),
            timestamp: Instant::now(),
            source: None,
            stack_trace: None,
        }
    }

    /// Add source information.
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// Convert to JSON.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","name":"{}","old":{},"new":{},"source":{}}}"#,
            self.id,
            self.name,
            self.old_value,
            self.new_value,
            self.source.as_ref().map(|s| format!("\"{}\"", s)).unwrap_or("null".to_string())
        )
    }
}

/// State history tracker.
pub struct StateHistory {
    changes: RefCell<Vec<StateChange>>,
    max_entries: usize,
    recording: RefCell<bool>,
}

impl StateHistory {
    /// Create a new state history tracker.
    pub fn new() -> Self {
        Self {
            changes: RefCell::new(Vec::new()),
            max_entries: 1000,
            recording: RefCell::new(true),
        }
    }

    /// Create with a custom max entries limit.
    pub fn with_max_entries(max: usize) -> Self {
        Self {
            changes: RefCell::new(Vec::new()),
            max_entries: max,
            recording: RefCell::new(true),
        }
    }

    /// Record a state change.
    pub fn record(&self, change: StateChange) {
        if !*self.recording.borrow() {
            return;
        }

        let mut changes = self.changes.borrow_mut();
        changes.push(change);

        // Trim if over limit
        if changes.len() > self.max_entries {
            changes.remove(0);
        }
    }

    /// Get all recorded changes.
    pub fn changes(&self) -> Vec<StateChange> {
        self.changes.borrow().clone()
    }

    /// Get changes for a specific state ID.
    pub fn changes_for(&self, id: &str) -> Vec<StateChange> {
        self.changes
            .borrow()
            .iter()
            .filter(|c| c.id == id)
            .cloned()
            .collect()
    }

    /// Get recent changes.
    pub fn recent(&self, count: usize) -> Vec<StateChange> {
        let changes = self.changes.borrow();
        let start = changes.len().saturating_sub(count);
        changes[start..].to_vec()
    }

    /// Clear all history.
    pub fn clear(&self) {
        self.changes.borrow_mut().clear();
    }

    /// Start recording.
    pub fn start(&self) {
        *self.recording.borrow_mut() = true;
    }

    /// Stop recording.
    pub fn stop(&self) {
        *self.recording.borrow_mut() = false;
    }

    /// Check if recording.
    pub fn is_recording(&self) -> bool {
        *self.recording.borrow()
    }

    /// Get change count.
    pub fn len(&self) -> usize {
        self.changes.borrow().len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.changes.borrow().is_empty()
    }

    /// Export history to JSON.
    pub fn to_json(&self) -> String {
        let changes: Vec<String> = self.changes.borrow().iter().map(|c| c.to_json()).collect();
        format!("[{}]", changes.join(","))
    }
}

impl Default for StateHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global state history.
thread_local! {
    static STATE_HISTORY: RefCell<StateHistory> = RefCell::new(StateHistory::new());
}

/// Record a state change globally.
pub fn record_state_change(change: StateChange) {
    STATE_HISTORY.with(|h| h.borrow().record(change));
}

/// Get global state history.
pub fn get_state_history() -> Vec<StateChange> {
    STATE_HISTORY.with(|h| h.borrow().changes())
}

/// Clear global state history.
pub fn clear_state_history() {
    STATE_HISTORY.with(|h| h.borrow().clear());
}

/// State snapshot for a point in time.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Snapshot ID.
    pub id: String,
    /// Label/description.
    pub label: String,
    /// Timestamp.
    pub timestamp: Instant,
    /// State values (key -> JSON value).
    pub state: HashMap<String, String>,
}

impl StateSnapshot {
    /// Create a new snapshot.
    pub fn new(label: &str) -> Self {
        Self {
            id: format!("snapshot-{}", uuid_v4()),
            label: label.to_string(),
            timestamp: Instant::now(),
            state: HashMap::new(),
        }
    }

    /// Add a state value.
    pub fn with_state(mut self, key: &str, value: &str) -> Self {
        self.state.insert(key.to_string(), value.to_string());
        self
    }

    /// Get a state value.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.state.get(key)
    }

    /// Convert to JSON.
    pub fn to_json(&self) -> String {
        let state_json: String = self
            .state
            .iter()
            .map(|(k, v)| format!("\"{}\":{}", k, v))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"id":"{}","label":"{}","state":{{{}}}}}"#,
            self.id, self.label, state_json
        )
    }
}

/// State diff between two snapshots.
#[derive(Debug, Clone)]
pub struct StateDiff {
    /// Keys that were added.
    pub added: Vec<String>,
    /// Keys that were removed.
    pub removed: Vec<String>,
    /// Keys that changed (key, old, new).
    pub changed: Vec<(String, String, String)>,
    /// Keys that remained the same.
    pub unchanged: Vec<String>,
}

impl StateDiff {
    /// Compare two snapshots.
    pub fn compare(old: &StateSnapshot, new: &StateSnapshot) -> Self {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();
        let mut unchanged = Vec::new();

        // Check for removed or changed keys
        for (key, old_value) in &old.state {
            if let Some(new_value) = new.state.get(key) {
                if old_value == new_value {
                    unchanged.push(key.clone());
                } else {
                    changed.push((key.clone(), old_value.clone(), new_value.clone()));
                }
            } else {
                removed.push(key.clone());
            }
        }

        // Check for added keys
        for key in new.state.keys() {
            if !old.state.contains_key(key) {
                added.push(key.clone());
            }
        }

        Self {
            added,
            removed,
            changed,
            unchanged,
        }
    }

    /// Check if there are any differences.
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.changed.is_empty()
    }

    /// Get a summary of changes.
    pub fn summary(&self) -> String {
        format!(
            "+{} -{} ~{} ={}",
            self.added.len(),
            self.removed.len(),
            self.changed.len(),
            self.unchanged.len()
        )
    }
}

/// State inspector for examining current application state.
pub struct StateInspector {
    snapshots: RefCell<Vec<StateSnapshot>>,
    max_snapshots: usize,
    watchers: RefCell<HashMap<String, Box<dyn Fn(&str, &str)>>>,
}

impl StateInspector {
    /// Create a new state inspector.
    pub fn new() -> Self {
        Self {
            snapshots: RefCell::new(Vec::new()),
            max_snapshots: 50,
            watchers: RefCell::new(HashMap::new()),
        }
    }

    /// Take a snapshot of current state.
    pub fn snapshot(&self, label: &str, state: HashMap<String, String>) -> StateSnapshot {
        let mut snapshot = StateSnapshot::new(label);
        snapshot.state = state;

        let mut snapshots = self.snapshots.borrow_mut();
        snapshots.push(snapshot.clone());

        if snapshots.len() > self.max_snapshots {
            snapshots.remove(0);
        }

        snapshot
    }

    /// Get all snapshots.
    pub fn snapshots(&self) -> Vec<StateSnapshot> {
        self.snapshots.borrow().clone()
    }

    /// Get the latest snapshot.
    pub fn latest(&self) -> Option<StateSnapshot> {
        self.snapshots.borrow().last().cloned()
    }

    /// Compare two snapshots by ID.
    pub fn compare(&self, old_id: &str, new_id: &str) -> Option<StateDiff> {
        let snapshots = self.snapshots.borrow();
        let old = snapshots.iter().find(|s| s.id == old_id)?;
        let new = snapshots.iter().find(|s| s.id == new_id)?;
        Some(StateDiff::compare(old, new))
    }

    /// Compare the last two snapshots.
    pub fn compare_latest(&self) -> Option<StateDiff> {
        let snapshots = self.snapshots.borrow();
        if snapshots.len() < 2 {
            return None;
        }
        let old = &snapshots[snapshots.len() - 2];
        let new = &snapshots[snapshots.len() - 1];
        Some(StateDiff::compare(old, new))
    }

    /// Add a state watcher.
    pub fn watch<F>(&self, key: &str, callback: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        self.watchers.borrow_mut().insert(key.to_string(), Box::new(callback));
    }

    /// Remove a state watcher.
    pub fn unwatch(&self, key: &str) {
        self.watchers.borrow_mut().remove(key);
    }

    /// Notify watchers of a state change.
    pub fn notify(&self, key: &str, old: &str, new: &str) {
        if let Some(callback) = self.watchers.borrow().get(key) {
            callback(old, new);
        }
    }

    /// Clear all snapshots.
    pub fn clear(&self) {
        self.snapshots.borrow_mut().clear();
    }
}

impl Default for StateInspector {
    fn default() -> Self {
        Self::new()
    }
}

/// State time travel for debugging.
pub struct StateTimeTravel {
    history: RefCell<Vec<StateSnapshot>>,
    current_index: RefCell<usize>,
}

impl StateTimeTravel {
    /// Create a new time travel instance.
    pub fn new() -> Self {
        Self {
            history: RefCell::new(Vec::new()),
            current_index: RefCell::new(0),
        }
    }

    /// Record a state checkpoint.
    pub fn checkpoint(&self, snapshot: StateSnapshot) {
        let mut history = self.history.borrow_mut();
        let mut index = self.current_index.borrow_mut();

        // Truncate any "future" states if we've gone back in time
        if *index < history.len() {
            history.truncate(*index);
        }

        history.push(snapshot);
        *index = history.len();
    }

    /// Go back one step.
    pub fn back(&self) -> Option<StateSnapshot> {
        let history = self.history.borrow();
        let mut index = self.current_index.borrow_mut();

        if *index > 1 {
            *index -= 1;
            return history.get(*index - 1).cloned();
        }
        None
    }

    /// Go forward one step.
    pub fn forward(&self) -> Option<StateSnapshot> {
        let history = self.history.borrow();
        let mut index = self.current_index.borrow_mut();

        if *index < history.len() {
            *index += 1;
            return history.get(*index - 1).cloned();
        }
        None
    }

    /// Go to a specific index.
    pub fn goto(&self, target: usize) -> Option<StateSnapshot> {
        let history = self.history.borrow();
        let mut index = self.current_index.borrow_mut();

        if target > 0 && target <= history.len() {
            *index = target;
            return history.get(target - 1).cloned();
        }
        None
    }

    /// Get current index.
    pub fn current_index(&self) -> usize {
        *self.current_index.borrow()
    }

    /// Get total history length.
    pub fn len(&self) -> usize {
        self.history.borrow().len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.history.borrow().is_empty()
    }

    /// Can go back?
    pub fn can_back(&self) -> bool {
        *self.current_index.borrow() > 1
    }

    /// Can go forward?
    pub fn can_forward(&self) -> bool {
        *self.current_index.borrow() < self.history.borrow().len()
    }

    /// Get current state.
    pub fn current(&self) -> Option<StateSnapshot> {
        let history = self.history.borrow();
        let index = *self.current_index.borrow();
        if index > 0 && index <= history.len() {
            return history.get(index - 1).cloned();
        }
        None
    }

    /// Clear history.
    pub fn clear(&self) {
        self.history.borrow_mut().clear();
        *self.current_index.borrow_mut() = 0;
    }
}

impl Default for StateTimeTravel {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple UUID v4 generator.
fn uuid_v4() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0) as u64;

    format!("{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (timestamp >> 32) as u32,
        (timestamp >> 16) as u16,
        (count >> 48) as u16 & 0x0fff,
        ((count >> 32) as u16 & 0x3fff) | 0x8000,
        count & 0xffffffffffff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_change() {
        let change = StateChange::new("sig-1", "count", "0", "1")
            .with_source("Counter::increment");

        assert_eq!(change.id, "sig-1");
        assert_eq!(change.old_value, "0");
        assert_eq!(change.new_value, "1");
        assert_eq!(change.source, Some("Counter::increment".to_string()));
    }

    #[test]
    fn test_state_history() {
        let history = StateHistory::new();

        history.record(StateChange::new("a", "count", "0", "1"));
        history.record(StateChange::new("b", "name", "\"old\"", "\"new\""));

        assert_eq!(history.len(), 2);

        let changes = history.changes_for("a");
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_state_snapshot() {
        let snapshot = StateSnapshot::new("initial")
            .with_state("count", "0")
            .with_state("name", "\"test\"");

        assert_eq!(snapshot.get("count"), Some(&"0".to_string()));
        assert_eq!(snapshot.label, "initial");
    }

    #[test]
    fn test_state_diff() {
        let old = StateSnapshot::new("old")
            .with_state("a", "1")
            .with_state("b", "2")
            .with_state("c", "3");

        let new = StateSnapshot::new("new")
            .with_state("a", "1")  // unchanged
            .with_state("b", "5")  // changed
            .with_state("d", "4"); // added (c removed)

        let diff = StateDiff::compare(&old, &new);

        assert!(diff.has_changes());
        assert_eq!(diff.added, vec!["d"]);
        assert_eq!(diff.removed, vec!["c"]);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.unchanged, vec!["a"]);
    }

    #[test]
    fn test_state_time_travel() {
        let tt = StateTimeTravel::new();

        tt.checkpoint(StateSnapshot::new("state-1").with_state("n", "1"));
        tt.checkpoint(StateSnapshot::new("state-2").with_state("n", "2"));
        tt.checkpoint(StateSnapshot::new("state-3").with_state("n", "3"));

        assert_eq!(tt.len(), 3);
        assert_eq!(tt.current_index(), 3);

        let back = tt.back().unwrap();
        assert_eq!(back.get("n"), Some(&"2".to_string()));
        assert_eq!(tt.current_index(), 2);

        let forward = tt.forward().unwrap();
        assert_eq!(forward.get("n"), Some(&"3".to_string()));
        assert_eq!(tt.current_index(), 3);
    }

    #[test]
    fn test_state_inspector() {
        let inspector = StateInspector::new();

        let mut state1 = HashMap::new();
        state1.insert("count".to_string(), "1".to_string());
        inspector.snapshot("first", state1);

        let mut state2 = HashMap::new();
        state2.insert("count".to_string(), "2".to_string());
        inspector.snapshot("second", state2);

        let diff = inspector.compare_latest().unwrap();
        assert!(diff.has_changes());
        assert_eq!(diff.changed.len(), 1);
    }
}
