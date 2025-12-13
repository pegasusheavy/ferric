//! Todo list components.

use ferric_ssr::prelude::*;
use serde::{Deserialize, Serialize};

/// A single todo item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    id: u32,
    text: String,
    completed: bool,
}

impl TodoItem {
    /// Create a new todo item.
    pub fn new(id: u32, text: impl Into<String>, completed: bool) -> Self {
        Self {
            id,
            text: text.into(),
            completed,
        }
    }
}

impl Renderable for TodoItem {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        let mut html = HtmlRenderer::new();

        let class = if self.completed {
            "todo-item completed"
        } else {
            "todo-item"
        };

        html.open_tag("li")
            .attr("class", class)
            .attr("data-id", &self.id.to_string())
            .close_open();

        // Checkbox
        html.open_tag("input")
            .attr("type", "checkbox")
            .bool_attr("checked", self.completed)
            .attr("style", "width: 20px; height: 20px; cursor: pointer;")
            .self_close();

        // Text
        let style = if self.completed {
            "flex: 1; text-decoration: line-through; color: #999;"
        } else {
            "flex: 1;"
        };

        html.open_tag("span").attr("style", style).close_open();
        html.text(&self.text);
        html.close_tag("span");

        // Delete button
        html.open_tag("button")
            .attr("data-action", "delete")
            .attr(
                "style",
                "background: none; border: none; color: #e74c3c; \
                 font-size: 1.2rem; cursor: pointer;",
            )
            .close_open();
        html.text("×");
        html.close_tag("button");

        html.close_tag("li");

        ctx.exit();
        Ok(html.finish())
    }
}

/// A list of todo items.
pub struct TodoList {
    items: Vec<TodoItem>,
}

impl TodoList {
    /// Create a new todo list with items.
    pub fn new(items: Vec<TodoItem>) -> Self {
        Self { items }
    }
}

impl Renderable for TodoList {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        // Add state for hydration
        ctx.add_state("todos", &self.items)?;

        let mut html = HtmlRenderer::new();

        html.open_tag("div")
            .attr("class", "todo-list-container")
            .close_open();

        html.element("h1", &[], "📝 Todo List");

        // Stats
        let total = self.items.len();
        let completed = self.items.iter().filter(|t| t.completed).count();
        let pending = total - completed;

        html.open_tag("div")
            .attr(
                "style",
                "display: flex; gap: 1rem; margin-bottom: 1rem; color: #666;",
            )
            .close_open();
        html.element(
            "span",
            &[],
            &format!("Total: {}", total),
        );
        html.element(
            "span",
            &[("style", "color: #27ae60;")],
            &format!("✓ Done: {}", completed),
        );
        html.element(
            "span",
            &[("style", "color: #f39c12;")],
            &format!("○ Pending: {}", pending),
        );
        html.close_tag("div");

        // Input for new todos
        html.open_tag("div")
            .attr(
                "style",
                "display: flex; gap: 0.5rem; margin-bottom: 1.5rem;",
            )
            .close_open();
        html.open_tag("input")
            .attr("type", "text")
            .attr("placeholder", "Add a new todo...")
            .attr("data-input", "new-todo")
            .attr(
                "style",
                "flex: 1; padding: 0.75rem; border: 2px solid #ddd; \
                 border-radius: 8px; font-size: 1rem;",
            )
            .self_close();
        html.open_tag("button")
            .attr("data-action", "add")
            .attr(
                "style",
                "padding: 0.75rem 1.5rem; background: #667eea; color: white; \
                 border: none; border-radius: 8px; cursor: pointer; font-weight: bold;",
            )
            .close_open();
        html.text("Add");
        html.close_tag("button");
        html.close_tag("div");

        // Todo list
        html.open_tag("ul")
            .attr("class", "todo-list")
            .attr(
                "style",
                "list-style: none; border: 1px solid #eee; border-radius: 8px; overflow: hidden;",
            )
            .close_open();

        for item in &self.items {
            let item_html = item.render(ctx)?;
            html.raw(&item_html);
        }

        html.close_tag("ul");

        // Clear completed button
        if completed > 0 {
            html.open_tag("button")
                .attr("data-action", "clear-completed")
                .attr(
                    "style",
                    "margin-top: 1rem; padding: 0.5rem 1rem; background: #e74c3c; \
                     color: white; border: none; border-radius: 6px; cursor: pointer;",
                )
                .close_open();
            html.text(&format!("Clear {} completed", completed));
            html.close_tag("button");
        }

        html.close_tag("div");

        ctx.exit();
        Ok(html.finish())
    }
}

