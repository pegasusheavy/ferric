//! SSR Demo - Demonstrating Ferric's Server-Side Rendering capabilities
//!
//! This example shows how to:
//! - Create renderable components
//! - Render components to HTML strings on the server
//! - Use platform detection for isomorphic code
//! - Serialize state for client-side hydration
//! - Build an SSR HTTP server with routes
//!
//! Run with: `cargo run -p ssr-demo`
//! Then visit: http://localhost:3000

mod components;

use components::{
    AboutPage, Counter, HomePage, Layout, NotFoundPage, TodoItem, TodoList, UserCard,
};
use ferric_ssr::prelude::*;
use ferric_ssr::render_to_string_with_state;
use hyper::body::Incoming;
use hyper::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Ferric SSR Demo");
    println!("==================");

    // Set platform to server mode
    set_platform(Platform::Server);

    // Create the SSR server
    let server = SsrServer::new(SsrConfig::new().bind("127.0.0.1:3000").dev(true))
        // Home page
        .get("/", handle_home)
        // About page
        .get("/about", handle_about)
        // Counter demo with state
        .get("/counter", handle_counter)
        // Todo list demo
        .get("/todos", handle_todos)
        // User profile demo
        .get("/user", handle_user)
        // API endpoint returning JSON
        .get("/api/data", handle_api)
        // 404 fallback
        .fallback(handle_not_found);

    println!();
    println!("Available routes:");
    println!("  GET /         - Home page");
    println!("  GET /about    - About page");
    println!("  GET /counter  - Counter with state serialization");
    println!("  GET /todos    - Todo list example");
    println!("  GET /user     - User profile card");
    println!("  GET /api/data - JSON API endpoint");
    println!();

    server.run().await?;

    Ok(())
}

/// Handle the home page
async fn handle_home(_req: Request<Incoming>) -> SsrResult<SsrResponse> {
    let page = HomePage::new(
        "Welcome to Ferric SSR",
        "Build fast, SEO-friendly web applications with Rust",
    );

    let layout = Layout::new("Home | Ferric SSR Demo", page);
    let html = render_to_string(&layout)?;

    Ok(SsrResponse::html(&html))
}

/// Handle the about page
async fn handle_about(_req: Request<Incoming>) -> SsrResult<SsrResponse> {
    let page = AboutPage::new();
    let layout = Layout::new("About | Ferric SSR Demo", page);
    let html = render_to_string(&layout)?;

    Ok(SsrResponse::html(&html))
}

/// Handle the counter page with state serialization
async fn handle_counter(_req: Request<Incoming>) -> SsrResult<SsrResponse> {
    // Create counter with initial state
    let counter = Counter::new(42);

    // Render with state for hydration
    let (content, state) = render_to_string_with_state(&counter)?;

    // Wrap in layout with hydration script
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Counter | Ferric SSR Demo</title>
    <style>{styles}</style>
</head>
<body>
    <nav><a href="/">← Back to Home</a></nav>
    <main>{content}</main>
    <script id="__FERRIC_STATE__" type="application/json">{state}</script>
    <script>
        // Client-side hydration - makes the counter interactive
        (function() {{
            // Parse the server-rendered state
            const stateEl = document.getElementById('__FERRIC_STATE__');
            const initialState = JSON.parse(stateEl.textContent);
            console.log('Hydrating counter with state:', initialState);

            // Current state (hydrated from server)
            let count = initialState.counter.value;
            const step = initialState.counter.step;

            // Get DOM elements
            const counterValue = document.querySelector('.counter-value');
            const decrementBtn = document.querySelector('[data-action="decrement"]');
            const incrementBtn = document.querySelector('[data-action="increment"]');

            // Update the display
            function updateDisplay() {{
                counterValue.textContent = count;
                // Add a little animation
                counterValue.style.transform = 'scale(1.1)';
                setTimeout(() => {{
                    counterValue.style.transform = 'scale(1)';
                }}, 100);
            }}

            // Add transition for smooth animation
            counterValue.style.transition = 'transform 0.1s ease';

            // Event listeners
            decrementBtn.addEventListener('click', () => {{
                count -= step;
                updateDisplay();
                console.log('Counter decremented to:', count);
            }});

            incrementBtn.addEventListener('click', () => {{
                count += step;
                updateDisplay();
                console.log('Counter incremented to:', count);
            }});

            // Add hover effects
            [decrementBtn, incrementBtn].forEach(btn => {{
                btn.style.transition = 'transform 0.1s ease, box-shadow 0.1s ease';
                btn.addEventListener('mouseenter', () => {{
                    btn.style.transform = 'scale(1.05)';
                    btn.style.boxShadow = '0 4px 12px rgba(0,0,0,0.2)';
                }});
                btn.addEventListener('mouseleave', () => {{
                    btn.style.transform = 'scale(1)';
                    btn.style.boxShadow = 'none';
                }});
            }});

            console.log('✅ Counter hydrated successfully!');
        }})();
    </script>
</body>
</html>"#,
        styles = get_styles(),
        content = content,
        state = state.as_json()
    );

    Ok(SsrResponse::html(&html))
}

/// Handle the todos page
async fn handle_todos(_req: Request<Incoming>) -> SsrResult<SsrResponse> {
    // Create todo items
    let todos = vec![
        TodoItem::new(1, "Learn Rust", true),
        TodoItem::new(2, "Build with Ferric", true),
        TodoItem::new(3, "Deploy SSR app", false),
        TodoItem::new(4, "Profit!", false),
    ];

    let todo_list = TodoList::new(todos);

    // Render with state for hydration
    let (content, state) = render_to_string_with_state(&todo_list)?;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Todos | Ferric SSR Demo</title>
    <style>{styles}
        .todo-item {{
            padding: 1rem;
            border-bottom: 1px solid #eee;
            display: flex;
            align-items: center;
            gap: 1rem;
            transition: background 0.2s;
        }}
        .todo-item:hover {{
            background: #f8f9fa;
        }}
        .todo-item.completed {{
            background: #f0fff0;
        }}
        .todo-item.completed span {{
            text-decoration: line-through;
            color: #999;
        }}
        .todo-item input[type="checkbox"] {{
            width: 20px;
            height: 20px;
            cursor: pointer;
        }}
        .todo-item button {{
            opacity: 0;
            transition: opacity 0.2s;
        }}
        .todo-item:hover button {{
            opacity: 1;
        }}
    </style>
</head>
<body>
    <nav style="padding: 1rem 2rem; background: rgba(255,255,255,0.1);">
        <a href="/" style="color: white; text-decoration: none; font-weight: 500;">← Back to Home</a>
    </nav>
    <main style="max-width: 800px; margin: 2rem auto; padding: 2rem; background: white; border-radius: 12px; box-shadow: 0 20px 40px rgba(0,0,0,0.2);">
        {content}
    </main>
    <script id="__FERRIC_STATE__" type="application/json">{state}</script>
    <script>
        // Client-side hydration for todo list
        (function() {{
            const stateEl = document.getElementById('__FERRIC_STATE__');
            let todos = JSON.parse(stateEl.textContent).todos;
            console.log('Hydrating todos with state:', todos);

            const todoList = document.querySelector('.todo-list');
            const newTodoInput = document.querySelector('[data-input="new-todo"]');
            const addBtn = document.querySelector('[data-action="add"]');

            function updateStats() {{
                const total = todos.length;
                const completed = todos.filter(t => t.completed).length;
                const pending = total - completed;

                // Find and update stats display
                const statsDiv = document.querySelector('.todo-list-container > div:nth-child(2)');
                if (statsDiv) {{
                    statsDiv.innerHTML = `
                        <span>Total: ${{total}}</span>
                        <span style="color: #27ae60;">✓ Done: ${{completed}}</span>
                        <span style="color: #f39c12;">○ Pending: ${{pending}}</span>
                    `;
                }}

                // Update clear completed button
                let clearBtn = document.querySelector('[data-action="clear-completed"]');
                if (completed > 0) {{
                    if (!clearBtn) {{
                        clearBtn = document.createElement('button');
                        clearBtn.setAttribute('data-action', 'clear-completed');
                        clearBtn.style.cssText = 'margin-top: 1rem; padding: 0.5rem 1rem; background: #e74c3c; color: white; border: none; border-radius: 6px; cursor: pointer;';
                        todoList.parentNode.appendChild(clearBtn);
                        clearBtn.addEventListener('click', clearCompleted);
                    }}
                    clearBtn.textContent = `Clear ${{completed}} completed`;
                }} else if (clearBtn) {{
                    clearBtn.remove();
                }}
            }}

            function renderTodo(todo) {{
                const li = document.createElement('li');
                li.className = 'todo-item' + (todo.completed ? ' completed' : '');
                li.setAttribute('data-id', todo.id);
                li.style.transition = 'transform 0.2s, opacity 0.2s';
                li.innerHTML = `
                    <input type="checkbox" ${{todo.completed ? 'checked' : ''}} style="width: 20px; height: 20px; cursor: pointer;">
                    <span class="todo-text" style="flex: 1; ${{todo.completed ? 'text-decoration: line-through; color: #999;' : ''}}">${{todo.text}}</span>
                    <button data-action="delete" style="background: none; border: none; color: #e74c3c; font-size: 1.2rem; cursor: pointer; opacity: 0; transition: opacity 0.2s;">×</button>
                `;
                // Show delete button on hover
                li.addEventListener('mouseenter', () => li.querySelector('button').style.opacity = '1');
                li.addEventListener('mouseleave', () => li.querySelector('button').style.opacity = '0');
                return li;
            }}

            function addTodo() {{
                const text = newTodoInput.value.trim();
                if (!text) return;

                const newId = Math.max(0, ...todos.map(t => t.id)) + 1;
                const newTodo = {{ id: newId, text, completed: false }};
                todos.push(newTodo);

                todoList.appendChild(renderTodo(newTodo));
                newTodoInput.value = '';
                updateStats();
                console.log('Added todo:', newTodo);
            }}

            function toggleTodo(id) {{
                const todo = todos.find(t => t.id === id);
                if (todo) {{
                    todo.completed = !todo.completed;
                    const li = todoList.querySelector(`[data-id="${{id}}"]`);
                    if (li) {{
                        // Update class and background
                        if (todo.completed) {{
                            li.classList.add('completed');
                            li.style.background = '#f0fff0';
                        }} else {{
                            li.classList.remove('completed');
                            li.style.background = '';
                        }}

                        // Update checkbox
                        const checkbox = li.querySelector('input[type="checkbox"]');
                        if (checkbox) checkbox.checked = todo.completed;

                        // Update text styling - find the span with the todo text
                        const textSpan = li.querySelector('span');
                        if (textSpan) {{
                            if (todo.completed) {{
                                textSpan.style.textDecoration = 'line-through';
                                textSpan.style.color = '#999';
                            }} else {{
                                textSpan.style.textDecoration = 'none';
                                textSpan.style.color = '#333';
                            }}
                            console.log('Updated text style for:', todo.text, 'completed:', todo.completed);
                        }}
                    }}
                    updateStats();
                    console.log('Toggled todo:', todo);
                }}
            }}

            function deleteTodo(id) {{
                todos = todos.filter(t => t.id !== id);
                const li = todoList.querySelector(`[data-id="${{id}}"]`);
                if (li) {{
                    li.style.transform = 'translateX(100%)';
                    li.style.opacity = '0';
                    setTimeout(() => li.remove(), 200);
                }}
                updateStats();
                console.log('Deleted todo:', id);
            }}

            function clearCompleted() {{
                const completedIds = todos.filter(t => t.completed).map(t => t.id);
                completedIds.forEach(id => {{
                    const li = todoList.querySelector(`[data-id="${{id}}"]`);
                    if (li) li.remove();
                }});
                todos = todos.filter(t => !t.completed);
                updateStats();
                console.log('Cleared completed todos');
            }}

            // Add transition styles and hover handlers to existing items
            todoList.querySelectorAll('.todo-item').forEach(li => {{
                li.style.transition = 'transform 0.2s, opacity 0.2s, background 0.2s';
                const deleteBtn = li.querySelector('[data-action="delete"]');
                if (deleteBtn) {{
                    deleteBtn.style.transition = 'opacity 0.2s';
                    li.addEventListener('mouseenter', () => deleteBtn.style.opacity = '1');
                    li.addEventListener('mouseleave', () => deleteBtn.style.opacity = '0');
                }}
            }});

            // Event delegation for todo items
            todoList.addEventListener('click', (e) => {{
                const li = e.target.closest('.todo-item');
                if (!li) return;
                const id = parseInt(li.getAttribute('data-id'));

                if (e.target.type === 'checkbox') {{
                    toggleTodo(id);
                }} else if (e.target.getAttribute('data-action') === 'delete') {{
                    deleteTodo(id);
                }}
            }});

            // Add new todo
            addBtn.addEventListener('click', addTodo);
            newTodoInput.addEventListener('keypress', (e) => {{
                if (e.key === 'Enter') addTodo();
            }});

            console.log('✅ Todo list hydrated successfully!');
        }})();
    </script>
</body>
</html>"#,
        styles = get_styles(),
        content = content,
        state = state.as_json()
    );

    Ok(SsrResponse::html(&html))
}

/// Handle the user profile page
async fn handle_user(_req: Request<Incoming>) -> SsrResult<SsrResponse> {
    // Simulate fetching user data
    let user = UserCard::new(
        "ferris",
        "Ferris the Crab",
        "ferris@rust-lang.org",
        "Rustacean | Systems Programming Enthusiast",
    );

    let layout = Layout::new("User Profile | Ferric SSR Demo", user);
    let html = render_to_string(&layout)?;

    Ok(SsrResponse::html(&html))
}

/// Handle API requests (returns JSON)
async fn handle_api(_req: Request<Incoming>) -> SsrResult<SsrResponse> {
    // Use platform detection to return different data
    let platform_info = if is_server() {
        "server"
    } else {
        "browser"
    };

    let data = serde_json::json!({
        "message": "Hello from Ferric SSR!",
        "platform": platform_info,
        "version": "0.1.0",
        "features": ["ssr", "hydration", "streaming"]
    });

    SsrResponse::json(&data)
}

/// Handle 404 not found
async fn handle_not_found(_req: Request<Incoming>) -> SsrResult<SsrResponse> {
    let page = NotFoundPage::new();
    let layout = Layout::new("404 Not Found | Ferric SSR Demo", page);
    let html = render_to_string(&layout)?;

    // Use HtmlResponse for custom 404 content
    Ok(SsrResponse::html(&format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 Not Found | Ferric SSR Demo</title>
    <style>{}</style>
</head>
<body style="display: flex; align-items: center; justify-content: center; min-height: 100vh;">
    {}
</body>
</html>"#,
        get_styles(),
        html
    )))
}

/// Get global styles
fn get_styles() -> &'static str {
    r#"
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            color: #1a1a2e;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        nav {
            padding: 1rem 2rem;
            background: rgba(255,255,255,0.1);
        }
        nav a {
            color: white;
            text-decoration: none;
            font-weight: 500;
        }
        nav a:hover { text-decoration: underline; }
        main {
            max-width: 800px;
            margin: 2rem auto;
            padding: 2rem;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.2);
        }
        h1 { color: #1a1a2e; margin-bottom: 1rem; }
        h2 { color: #4a4a6a; margin-bottom: 0.5rem; }
        p { color: #666; margin-bottom: 1rem; }
        .counter {
            text-align: center;
            padding: 2rem;
        }
        .counter-value {
            font-size: 4rem;
            font-weight: bold;
            color: #667eea;
        }
        .todo-list { list-style: none; }
        .todo-item {
            padding: 1rem;
            border-bottom: 1px solid #eee;
            display: flex;
            align-items: center;
            gap: 1rem;
        }
        .todo-item.completed span {
            text-decoration: line-through;
            color: #999;
        }
        .user-card {
            text-align: center;
            padding: 2rem;
        }
        .avatar {
            width: 120px;
            height: 120px;
            border-radius: 50%;
            background: linear-gradient(135deg, #667eea, #764ba2);
            margin: 0 auto 1rem;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 3rem;
            color: white;
        }
        .features { margin-top: 2rem; }
        .feature {
            background: #f8f9fa;
            padding: 1rem;
            border-radius: 8px;
            margin-bottom: 1rem;
        }
        .feature h3 { color: #667eea; }
        code {
            background: #f0f0f0;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-family: monospace;
        }
    "#
}
