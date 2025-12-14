//! Reactive store for the todo application.
//!
//! Uses Ferric's reactive system for fine-grained state management.

use crate::models::{Todo, TodoFilter, TodoStats};
use crate::services::TodoStorage;
use ferric_core::reactive::{
    batch, computed, effect, signal, Computed, Effect, Signal,
};
use std::rc::Rc;

/// The reactive store for the todo application.
///
/// All state is managed through signals, with computed values for derived
/// state and effects for side effects.
#[derive(Clone)]
pub struct TodoStore {
    /// All todos.
    pub todos: Signal<Vec<Todo>>,
    /// Current filter.
    pub filter: Signal<TodoFilter>,
    /// New todo input text.
    pub new_todo_text: Signal<String>,
    /// ID of the todo being edited (None if not editing).
    pub editing_id: Signal<Option<u32>>,
    /// Edit text buffer.
    pub edit_text: Signal<String>,
    /// Next todo ID.
    next_id: Signal<u32>,

    // Computed values
    /// Filtered todos based on current filter.
    pub filtered_todos: Computed<Vec<Todo>>,
    /// Statistics about todos.
    pub stats: Computed<TodoStats>,
    /// Whether all todos are completed.
    pub all_completed: Computed<bool>,
    /// Whether there are any completed todos.
    pub has_completed: Computed<bool>,
    /// Whether the list is empty.
    pub is_empty: Computed<bool>,
    /// Whether the main section should be visible.
    pub show_main: Computed<bool>,
    /// Whether the footer should be visible.
    pub show_footer: Computed<bool>,
    /// Active count text (e.g., "1 item left" or "3 items left").
    pub items_left_text: Computed<String>,

    // Effects (stored to prevent dropping)
    _persistence_effect: Rc<Effect>,
}

impl TodoStore {
    /// Create a new store, loading initial state from localStorage.
    pub fn new() -> Self {
        // Load initial state
        let initial_todos = TodoStorage::load();
        let next_id = initial_todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;

        // Create signals
        let todos = signal(initial_todos);
        let filter = signal(TodoFilter::All);
        let new_todo_text = signal(String::new());
        let editing_id = signal(None);
        let edit_text = signal(String::new());
        let next_id_signal = signal(next_id);

        // Create computed values
        let todos_for_filter = todos.clone();
        let filter_for_computed = filter.clone();
        let filtered_todos = computed(move || {
            let todos = todos_for_filter.get();
            let filter = filter_for_computed.get();
            todos.into_iter().filter(|t| filter.matches(t)).collect()
        });

        let todos_for_stats = todos.clone();
        let stats = computed(move || {
            TodoStats::from_todos(&todos_for_stats.get())
        });

        let todos_for_all = todos.clone();
        let all_completed = computed(move || {
            let todos = todos_for_all.get();
            !todos.is_empty() && todos.iter().all(|t| t.completed)
        });

        let stats_for_has = stats.clone();
        let has_completed = computed(move || {
            stats_for_has.get().completed > 0
        });

        let todos_for_empty = todos.clone();
        let is_empty = computed(move || {
            todos_for_empty.get().is_empty()
        });

        let is_empty_for_main = is_empty.clone();
        let show_main = computed(move || {
            !is_empty_for_main.get()
        });

        let is_empty_for_footer = is_empty.clone();
        let show_footer = computed(move || {
            !is_empty_for_footer.get()
        });

        let stats_for_text = stats.clone();
        let items_left_text = computed(move || {
            let stats = stats_for_text.get();
            let word = if stats.active == 1 { "item" } else { "items" };
            format!("{} {} left", stats.active, word)
        });

        // Create persistence effect
        let todos_for_persist = todos.clone();
        let persistence_effect = effect(move || {
            let todos = todos_for_persist.get();
            TodoStorage::save(&todos);
        });

        Self {
            todos,
            filter,
            new_todo_text,
            editing_id,
            edit_text,
            next_id: next_id_signal,
            filtered_todos,
            stats,
            all_completed,
            has_completed,
            is_empty,
            show_main,
            show_footer,
            items_left_text,
            _persistence_effect: Rc::new(persistence_effect),
        }
    }

    // ==================== Actions ====================

    /// Add a new todo from the input text.
    pub fn add_todo(&self) {
        let text = self.new_todo_text.get().trim().to_string();
        if text.is_empty() {
            return;
        }

        batch(|| {
            let id = self.next_id.get();
            self.next_id.update(|n| n + 1);

            self.todos.mutate(|todos| {
                todos.push(Todo::new(id, text));
            });

            self.new_todo_text.set(String::new());
        });
    }

    /// Toggle a todo's completed state.
    pub fn toggle_todo(&self, id: u32) {
        self.todos.mutate(|todos| {
            if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
                todo.toggle();
            }
        });
    }

    /// Delete a todo.
    pub fn delete_todo(&self, id: u32) {
        self.todos.mutate(|todos| {
            todos.retain(|t| t.id != id);
        });
    }

    /// Toggle all todos.
    pub fn toggle_all(&self) {
        let all_completed = self.all_completed.get();
        self.todos.mutate(|todos| {
            for todo in todos.iter_mut() {
                todo.completed = !all_completed;
            }
        });
    }

    /// Clear all completed todos.
    pub fn clear_completed(&self) {
        self.todos.mutate(|todos| {
            todos.retain(|t| !t.completed);
        });
    }

    /// Set the current filter.
    pub fn set_filter(&self, filter: TodoFilter) {
        self.filter.set(filter);
    }

    /// Start editing a todo.
    pub fn start_editing(&self, id: u32) {
        if let Some(todo) = self.todos.get().iter().find(|t| t.id == id) {
            batch(|| {
                self.editing_id.set(Some(id));
                self.edit_text.set(todo.text.clone());
            });
        }
    }

    /// Commit the edit.
    pub fn commit_edit(&self) {
        if let Some(id) = self.editing_id.get() {
            let text = self.edit_text.get().trim().to_string();

            batch(|| {
                if text.is_empty() {
                    // Empty text = delete
                    self.delete_todo(id);
                } else {
                    self.todos.mutate(|todos| {
                        if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
                            todo.text = text;
                        }
                    });
                }
                self.editing_id.set(None);
            });
        }
    }

    /// Cancel the edit.
    pub fn cancel_edit(&self) {
        batch(|| {
            self.editing_id.set(None);
            self.edit_text.set(String::new());
        });
    }

    /// Update the new todo input text.
    pub fn set_new_todo_text(&self, text: String) {
        self.new_todo_text.set(text);
    }

    /// Update the edit text.
    pub fn set_edit_text(&self, text: String) {
        self.edit_text.set(text);
    }

    /// Check if a specific todo is being edited.
    pub fn is_editing(&self, id: u32) -> bool {
        self.editing_id.get() == Some(id)
    }
}

impl Default for TodoStore {
    fn default() -> Self {
        Self::new()
    }
}

