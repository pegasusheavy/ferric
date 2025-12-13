//! Data models for the todo application.

use serde::{Deserialize, Serialize};

/// A single todo item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    /// Unique identifier.
    pub id: u32,
    /// The todo text.
    pub text: String,
    /// Whether the todo is completed.
    pub completed: bool,
}

impl Todo {
    /// Create a new todo item.
    pub fn new(id: u32, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
            completed: false,
        }
    }

    /// Toggle the completed state.
    pub fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}

/// Filter for displaying todos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TodoFilter {
    /// Show all todos.
    #[default]
    All,
    /// Show only active (incomplete) todos.
    Active,
    /// Show only completed todos.
    Completed,
}

impl TodoFilter {
    /// Check if a todo matches this filter.
    pub fn matches(&self, todo: &Todo) -> bool {
        match self {
            TodoFilter::All => true,
            TodoFilter::Active => !todo.completed,
            TodoFilter::Completed => todo.completed,
        }
    }

    /// Get the filter name for display.
    pub fn name(&self) -> &'static str {
        match self {
            TodoFilter::All => "All",
            TodoFilter::Active => "Active",
            TodoFilter::Completed => "Completed",
        }
    }
}

/// Statistics about the todo list.
#[derive(Debug, Clone, Default)]
pub struct TodoStats {
    /// Total number of todos.
    pub total: usize,
    /// Number of completed todos.
    pub completed: usize,
    /// Number of active todos.
    pub active: usize,
}

impl TodoStats {
    /// Calculate stats from a list of todos.
    pub fn from_todos(todos: &[Todo]) -> Self {
        let total = todos.len();
        let completed = todos.iter().filter(|t| t.completed).count();
        let active = total - completed;

        Self { total, completed, active }
    }
}

