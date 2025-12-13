# Structural Directives

Comprehensive guide to structural directives in Ferric.

## Overview

Structural directives modify the DOM structure by adding, removing, or manipulating elements. Ferric provides three built-in structural directives:

1. **`*if`** - Conditionally renders content
2. **`*for`** - Renders a template for each item in a collection
3. **`*switch`** - Conditionally renders one of several templates

## *if Directive

The `*if` directive conditionally includes a template based on a boolean condition.

### Features

- Lazy rendering (only creates DOM when condition is true)
- Proper cleanup when condition becomes false
- Efficient updates (doesn't recreate DOM unnecessarily)

### Usage

```rust
use ferric::directives::IfDirective;

let mut directive = IfDirective::new();
directive.set_template(template_element);
directive.set_container(container_element);
directive.set_condition(true); // Show content
directive.set_condition(false); // Hide content
```

### Template Syntax

```html
<div *if="isVisible">
  This content is conditionally rendered
</div>
```

## *for Directive

The `*for` directive renders a template for each item in a collection, efficiently managing views through a diffing algorithm.

### Features

- **Efficient Diffing**: Only updates changed items
- **Track-by Support**: Customize how items are identified
- **View Reuse**: Reuses DOM elements for unchanged items
- **Context Variables**: Access item, index, and more
- **Memory Safe**: Properly cleans up on drop

### Context Variables

Each rendered item has access to the following context:

| Variable | Type | Description |
|----------|------|-------------|
| `item` | `T` | The current item |
| `index` | `usize` | Zero-based index (0, 1, 2, ...) |
| `count` | `usize` | Total number of items |
| `first` | `bool` | True if this is the first item |
| `last` | `bool` | True if this is the last item |
| `even` | `bool` | True if index is even |
| `odd` | `bool` | True if index is odd |

### Usage

```rust
use ferric::directives::ForDirective;

#[derive(Clone)]
struct User {
    id: String,
    name: String,
}

let mut directive = ForDirective::new();
directive.set_template(template_element);
directive.set_container(container_element);

// Set track-by for efficient updates
directive.track_by(|user| user.id.clone());

// Set items
let users = vec![
    User { id: "1".to_string(), name: "Alice".to_string() },
    User { id: "2".to_string(), name: "Bob".to_string() },
];
directive.set_items(users);
```

### Template Syntax

```html
<ul>
  <li *for="user of users; trackBy: userId">
    {{ user.name }}
  </li>
</ul>
```

### Performance Characteristics

| Operation | Without track-by | With track-by |
|-----------|------------------|---------------|
| Initial render | O(n) | O(n) |
| Update all items | O(n) | O(n) |
| Add 1 item | O(n) | O(1) |
| Remove 1 item | O(n) | O(1) |
| Reorder items | O(n) | O(n) with view reuse |

**Recommendation**: Always use `track_by` for dynamic lists!

### Advanced Example

```rust
#[derive(Clone)]
struct Todo {
    id: usize,
    title: String,
    completed: bool,
}

let mut directive = ForDirective::new();
directive.set_template(todo_template);
directive.set_container(todo_list);
directive.track_by(|todo| todo.id.to_string());

let todos = vec![
    Todo { id: 1, title: "Learn Rust".to_string(), completed: true },
    Todo { id: 2, title: "Build app".to_string(), completed: false },
];
directive.set_items(todos);

// Later, update the list
directive.set_items(vec![
    // Item 1 unchanged - view reused!
    Todo { id: 1, title: "Learn Rust".to_string(), completed: true },
    // Item 2 removed - view destroyed
    // Item 3 added - new view created
    Todo { id: 3, title: "Deploy".to_string(), completed: false },
]);
```

## *switch Directive

The `*switch` directive conditionally renders one of several templates based on a value.

### Features

- Efficient switching (only one case rendered at a time)
- Default case support
- Type-safe value matching

### Usage

```rust
use ferric::directives::SwitchDirective;

let mut directive = SwitchDirective::new();
directive.set_container(container_element);

// Add cases
directive.add_case("loading", loading_template);
directive.add_case("success", success_template);
directive.add_case("error", error_template);
directive.set_default(default_template);

// Set the active case
directive.set_value("loading");
```

### Template Syntax

```html
<div [switch]="state">
  <div *case="'loading'">Loading...</div>
  <div *case="'success'">Success!</div>
  <div *case="'error'">Error occurred</div>
  <div *default>Unknown state</div>
</div>
```

## Implementation Details

### View Lifecycle

All directives follow a consistent view lifecycle:

1. **Creation**: Template is cloned when needed
2. **Insertion**: View is appended to container
3. **Update**: Context is updated, bindings refreshed
4. **Removal**: View is removed from DOM
5. **Cleanup**: Resources freed on directive drop

### Diffing Algorithm (ForDirective)

The `ForDirective` uses an efficient diffing algorithm:

```rust
fn update() {
    // 1. Build map of new items by key
    let new_items = map_items_by_key(&self.items);

    // 2. Identify views to keep/remove
    for view in old_views {
        if new_items.contains_key(&view.key) {
            // Reuse this view
        } else {
            // Remove this view
        }
    }

    // 3. Create views for new items
    for item in new_items {
        if !view_exists(item.key) {
            // Create new view
        }
    }

    // 4. Reorder views in DOM
    // (Efficiently moves existing nodes)
}
```

### Memory Management

- All directives implement `Drop` for cleanup
- Views are removed from DOM when directive is dropped
- No memory leaks from orphaned DOM nodes

## Best Practices

### 1. Always Use track_by for Dynamic Lists

```rust
// ❌ Bad - tracks by index
directive.set_items(items);

// ✅ Good - tracks by unique ID
directive.track_by(|item| item.id.to_string());
directive.set_items(items);
```

### 2. Minimize Template Complexity

```html
<!-- ❌ Bad - complex nested structure in *for -->
<div *for="item of items">
  <div>
    <div>
      <complex-component [data]="item"></complex-component>
    </div>
  </div>
</div>

<!-- ✅ Good - simple wrapper -->
<item-component *for="item of items" [item]="item"></item-component>
```

### 3. Use *switch for Multiple Conditions

```html
<!-- ❌ Bad - multiple *if -->
<div *if="state === 'loading'">Loading...</div>
<div *if="state === 'success'">Success!</div>
<div *if="state === 'error'">Error!</div>

<!-- ✅ Good - *switch -->
<div [switch]="state">
  <div *case="'loading'">Loading...</div>
  <div *case="'success'">Success!</div>
  <div *case="'error'">Error!</div>
</div>
```

### 4. Leverage Context Variables

```html
<ul>
  <li *for="item of items; let i = index; let first = first">
    <span *if="first" class="badge">First!</span>
    {{ i + 1 }}. {{ item.name }}
  </li>
</ul>
```

## Performance Tips

### *for Directive

1. **Use track_by**: Essential for good performance
2. **Keep templates simple**: Complex templates slow rendering
3. **Limit list size**: Consider virtual scrolling for large lists
4. **Batch updates**: Update items in one call, not individually

### *if Directive

1. **Avoid toggling frequently**: Each toggle modifies DOM
2. **Use CSS classes for visibility**: If content is expensive to create
3. **Lazy load heavy content**: Wait until *if condition is true

### *switch Directive

1. **Minimize case count**: Too many cases can slow initialization
2. **Use default sparingly**: Only if truly needed
3. **Combine with lazy loading**: Load templates on demand

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_directive_tracks_items() {
        let mut directive = ForDirective::new();

        // Set initial items
        directive.set_items(vec![1, 2, 3]);
        assert_eq!(directive.view_count(), 3);

        // Update items
        directive.set_items(vec![2, 3, 4]);
        assert_eq!(directive.view_count(), 3);

        // Clear items
        directive.set_items(vec![]);
        assert_eq!(directive.view_count(), 0);
    }

    #[test]
    fn test_if_directive_conditions() {
        let mut directive = IfDirective::new();

        // Initially hidden
        assert!(!directive.condition());

        // Show
        directive.set_condition(true);
        assert!(directive.condition());

        // Hide
        directive.set_condition(false);
        assert!(!directive.condition());
    }
}
```

## Further Reading

- [Angular Structural Directives](https://angular.io/guide/structural-directives)
- [Virtual DOM Diffing](https://reactjs.org/docs/reconciliation.html)
- [Efficient List Rendering](https://vuejs.org/guide/essentials/list.html#maintaining-state-with-key)

---

For implementation details, see `ferric-core/src/directives/structural.rs`.

