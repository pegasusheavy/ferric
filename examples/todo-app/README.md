# Ferric Todo App

A fully reactive todo list application demonstrating Ferric's signals-based state management.

## Features

- ✅ **Reactive Signals** - All state managed with `Signal<T>`
- ✅ **Computed Values** - Derived state with automatic dependency tracking
- ✅ **Effects** - Side effects for DOM updates and localStorage persistence
- ✅ **Batching** - Multiple updates batched for efficiency
- ✅ **localStorage Persistence** - Todos saved automatically

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          TodoStore                               │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐  │
│  │     Signals      │  │    Computed      │  │    Effects    │  │
│  │  ───────────────│  │  ───────────────│  │  ────────────│  │
│  │  todos          │──│  filtered_todos  │──│  persistence │  │
│  │  filter         │  │  stats           │  │  DOM updates │  │
│  │  new_todo_text  │  │  all_completed   │  │              │  │
│  │  editing_id     │  │  has_completed   │  │              │  │
│  │  edit_text      │  │  items_left_text │  │              │  │
│  └──────────────────┘  └──────────────────┘  └───────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Reactive Flow

1. **User Action** → Updates a Signal
2. **Signal Change** → Triggers dependent Computed values
3. **Computed Change** → Triggers subscribed Effects
4. **Effects** → Update DOM / Save to localStorage

```rust
// Example: Adding a todo
pub fn add_todo(&self) {
    let text = self.new_todo_text.get().trim().to_string();
    if text.is_empty() { return; }

    // Batch multiple updates
    batch(|| {
        // Update next_id signal
        let id = self.next_id.get();
        self.next_id.update(|n| n + 1);

        // Update todos signal (triggers computed & effects)
        self.todos.mutate(|todos| {
            todos.push(Todo::new(id, text));
        });

        // Clear input
        self.new_todo_text.set(String::new());
    });
    // After batch: all effects run once
}
```

## Key Concepts Demonstrated

### Signals (Reactive State)

```rust
// Primary state
pub todos: Signal<Vec<Todo>>,
pub filter: Signal<TodoFilter>,
pub new_todo_text: Signal<String>,
pub editing_id: Signal<Option<u32>>,
```

### Computed Values (Derived State)

```rust
// Automatically updates when todos or filter change
let filtered_todos = computed(move || {
    let todos = todos.get();
    let filter = filter.get();
    todos.into_iter().filter(|t| filter.matches(t)).collect()
});

// Automatically updates when stats change
let items_left_text = computed(move || {
    let stats = stats.get();
    let word = if stats.active == 1 { "item" } else { "items" };
    format!("{} {} left", stats.active, word)
});
```

### Effects (Side Effects)

```rust
// Persistence effect - runs whenever todos change
let persistence_effect = effect(move || {
    let todos = todos.get();
    TodoStorage::save(&todos);
});

// DOM effect - updates todo count display
let fx = effect(move || {
    let text = store.items_left_text.get();
    set_text(&count_el, &text);
});
```

### Batching (Performance)

```rust
// Multiple signal updates batched into one notification
batch(|| {
    self.editing_id.set(None);
    self.edit_text.set(String::new());
}); // Effects run once after batch
```

## File Structure

```
src/
├── lib.rs          # Entry point & WASM setup
├── app.rs          # Main application with reactive bindings
├── store.rs        # TodoStore with signals, computed, effects
├── models.rs       # Todo, TodoFilter, TodoStats
├── services.rs     # TodoStorage (localStorage)
├── template.rs     # DOM utilities
└── components/
    ├── todo_app.css    # Styles
    └── ...
```

## Building

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build for web
wasm-pack build --target web

# Or use the build tool
cd ferric_build && pnpm install && pnpm build
```

## Running

Serve the `examples/todo-app` directory with any static file server:

```bash
# Using Python
python -m http.server 8080

# Using Node.js
npx serve .

# Using Rust
cargo install miniserve && miniserve . --port 8080
```

Then open http://localhost:8080 in your browser.

## Comparison with Angular

| Angular | Ferric |
|---------|--------|
| `@Input() todos` | `todos: Signal<Vec<Todo>>` |
| `{{ todos \| filter }}` | `filtered_todos: Computed<Vec<Todo>>` |
| `(click)="toggle()"` | `on_event(&el, "click", \|_\| store.toggle())` |
| `*ngIf="show"` | `effect(\|\| { el.style.display = if show.get() {...} })` |
| `*ngFor="let t of todos"` | Reactive list rendering with `effect` |
| `localStorage.setItem()` | `effect(\|\| TodoStorage::save(&todos.get()))` |

## License

MIT
