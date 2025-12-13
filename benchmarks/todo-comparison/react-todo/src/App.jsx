import React, { useState, useCallback, useMemo, useEffect } from 'react';

let nextId = 1;

function App() {
  const [todos, setTodos] = useState([]);
  const [filter, setFilter] = useState('all');
  const [newTodo, setNewTodo] = useState('');

  useEffect(() => {
    window.__BENCHMARK__?.mark('app-mounted');
  }, []);

  // Add a single todo
  const addTodo = useCallback(() => {
    if (!newTodo.trim()) return;

    window.__BENCHMARK__?.mark('add-start');
    setTodos(prev => [...prev, {
      id: nextId++,
      text: newTodo.trim(),
      completed: false,
      createdAt: Date.now(),
    }]);
    setNewTodo('');
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('add-end');
    });
  }, [newTodo]);

  // Bulk add todos (for benchmarking)
  const bulkAdd = useCallback((count) => {
    window.__BENCHMARK__?.mark('bulk-add-start');
    const newTodos = Array.from({ length: count }, (_, i) => ({
      id: nextId++,
      text: `Bulk Todo ${i + 1}`,
      completed: false,
      createdAt: Date.now(),
    }));
    setTodos(prev => [...prev, ...newTodos]);
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('bulk-add-end');
    });
  }, []);

  // Toggle single todo
  const toggleTodo = useCallback((id) => {
    window.__BENCHMARK__?.mark('toggle-start');
    setTodos(prev => prev.map(todo =>
      todo.id === id ? { ...todo, completed: !todo.completed } : todo
    ));
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('toggle-end');
    });
  }, []);

  // Toggle all todos
  const toggleAll = useCallback(() => {
    window.__BENCHMARK__?.mark('toggle-all-start');
    setTodos(prev => {
      const allCompleted = prev.every(t => t.completed);
      return prev.map(todo => ({ ...todo, completed: !allCompleted }));
    });
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('toggle-all-end');
    });
  }, []);

  // Delete single todo
  const deleteTodo = useCallback((id) => {
    window.__BENCHMARK__?.mark('delete-start');
    setTodos(prev => prev.filter(todo => todo.id !== id));
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('delete-end');
    });
  }, []);

  // Clear completed
  const clearCompleted = useCallback(() => {
    window.__BENCHMARK__?.mark('clear-start');
    setTodos(prev => prev.filter(todo => !todo.completed));
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('clear-end');
    });
  }, []);

  // Filtered todos
  const filteredTodos = useMemo(() => {
    window.__BENCHMARK__?.mark('filter-start');
    const result = todos.filter(todo => {
      if (filter === 'active') return !todo.completed;
      if (filter === 'completed') return todo.completed;
      return true;
    });
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('filter-end');
    });
    return result;
  }, [todos, filter]);

  // Stats
  const stats = useMemo(() => ({
    total: todos.length,
    active: todos.filter(t => !t.completed).length,
    completed: todos.filter(t => t.completed).length,
  }), [todos]);

  // Expose benchmark functions to window
  useEffect(() => {
    window.__APP__ = {
      addTodo: (text) => {
        setNewTodo(text);
        setTimeout(() => {
          document.querySelector('[data-action="add"]')?.click();
        }, 0);
      },
      bulkAdd,
      toggleTodo,
      toggleAll,
      deleteTodo,
      clearCompleted,
      setFilter,
      getTodos: () => todos,
      getStats: () => stats,
    };
  }, [bulkAdd, toggleTodo, toggleAll, deleteTodo, clearCompleted, todos, stats]);

  return (
    <div className="todo-app">
      <header className="header">
        <h1>React Todos</h1>
        <div className="stats">
          <span>{stats.total} total</span>
          <span>{stats.active} active</span>
          <span>{stats.completed} completed</span>
        </div>
      </header>

      <section className="input-section">
        <input
          type="text"
          className="new-todo"
          placeholder="What needs to be done?"
          value={newTodo}
          onChange={(e) => setNewTodo(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && addTodo()}
        />
        <button data-action="add" onClick={addTodo}>Add</button>
      </section>

      <section className="controls">
        <button onClick={toggleAll}>Toggle All</button>
        <button onClick={() => bulkAdd(100)}>Add 100</button>
        <button onClick={() => bulkAdd(1000)}>Add 1000</button>
        <button onClick={clearCompleted}>Clear Completed</button>
      </section>

      <section className="filters">
        <button
          className={filter === 'all' ? 'active' : ''}
          onClick={() => setFilter('all')}
        >
          All
        </button>
        <button
          className={filter === 'active' ? 'active' : ''}
          onClick={() => setFilter('active')}
        >
          Active
        </button>
        <button
          className={filter === 'completed' ? 'active' : ''}
          onClick={() => setFilter('completed')}
        >
          Completed
        </button>
      </section>

      <ul className="todo-list">
        {filteredTodos.map(todo => (
          <li key={todo.id} className={todo.completed ? 'completed' : ''}>
            <input
              type="checkbox"
              checked={todo.completed}
              onChange={() => toggleTodo(todo.id)}
            />
            <span className="todo-text">{todo.text}</span>
            <button className="delete" onClick={() => deleteTodo(todo.id)}>×</button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;



let nextId = 1;

function App() {
  const [todos, setTodos] = useState([]);
  const [filter, setFilter] = useState('all');
  const [newTodo, setNewTodo] = useState('');

  useEffect(() => {
    window.__BENCHMARK__?.mark('app-mounted');
  }, []);

  // Add a single todo
  const addTodo = useCallback(() => {
    if (!newTodo.trim()) return;

    window.__BENCHMARK__?.mark('add-start');
    setTodos(prev => [...prev, {
      id: nextId++,
      text: newTodo.trim(),
      completed: false,
      createdAt: Date.now(),
    }]);
    setNewTodo('');
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('add-end');
    });
  }, [newTodo]);

  // Bulk add todos (for benchmarking)
  const bulkAdd = useCallback((count) => {
    window.__BENCHMARK__?.mark('bulk-add-start');
    const newTodos = Array.from({ length: count }, (_, i) => ({
      id: nextId++,
      text: `Bulk Todo ${i + 1}`,
      completed: false,
      createdAt: Date.now(),
    }));
    setTodos(prev => [...prev, ...newTodos]);
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('bulk-add-end');
    });
  }, []);

  // Toggle single todo
  const toggleTodo = useCallback((id) => {
    window.__BENCHMARK__?.mark('toggle-start');
    setTodos(prev => prev.map(todo =>
      todo.id === id ? { ...todo, completed: !todo.completed } : todo
    ));
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('toggle-end');
    });
  }, []);

  // Toggle all todos
  const toggleAll = useCallback(() => {
    window.__BENCHMARK__?.mark('toggle-all-start');
    setTodos(prev => {
      const allCompleted = prev.every(t => t.completed);
      return prev.map(todo => ({ ...todo, completed: !allCompleted }));
    });
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('toggle-all-end');
    });
  }, []);

  // Delete single todo
  const deleteTodo = useCallback((id) => {
    window.__BENCHMARK__?.mark('delete-start');
    setTodos(prev => prev.filter(todo => todo.id !== id));
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('delete-end');
    });
  }, []);

  // Clear completed
  const clearCompleted = useCallback(() => {
    window.__BENCHMARK__?.mark('clear-start');
    setTodos(prev => prev.filter(todo => !todo.completed));
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('clear-end');
    });
  }, []);

  // Filtered todos
  const filteredTodos = useMemo(() => {
    window.__BENCHMARK__?.mark('filter-start');
    const result = todos.filter(todo => {
      if (filter === 'active') return !todo.completed;
      if (filter === 'completed') return todo.completed;
      return true;
    });
    requestAnimationFrame(() => {
      window.__BENCHMARK__?.mark('filter-end');
    });
    return result;
  }, [todos, filter]);

  // Stats
  const stats = useMemo(() => ({
    total: todos.length,
    active: todos.filter(t => !t.completed).length,
    completed: todos.filter(t => t.completed).length,
  }), [todos]);

  // Expose benchmark functions to window
  useEffect(() => {
    window.__APP__ = {
      addTodo: (text) => {
        setNewTodo(text);
        setTimeout(() => {
          document.querySelector('[data-action="add"]')?.click();
        }, 0);
      },
      bulkAdd,
      toggleTodo,
      toggleAll,
      deleteTodo,
      clearCompleted,
      setFilter,
      getTodos: () => todos,
      getStats: () => stats,
    };
  }, [bulkAdd, toggleTodo, toggleAll, deleteTodo, clearCompleted, todos, stats]);

  return (
    <div className="todo-app">
      <header className="header">
        <h1>React Todos</h1>
        <div className="stats">
          <span>{stats.total} total</span>
          <span>{stats.active} active</span>
          <span>{stats.completed} completed</span>
        </div>
      </header>

      <section className="input-section">
        <input
          type="text"
          className="new-todo"
          placeholder="What needs to be done?"
          value={newTodo}
          onChange={(e) => setNewTodo(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && addTodo()}
        />
        <button data-action="add" onClick={addTodo}>Add</button>
      </section>

      <section className="controls">
        <button onClick={toggleAll}>Toggle All</button>
        <button onClick={() => bulkAdd(100)}>Add 100</button>
        <button onClick={() => bulkAdd(1000)}>Add 1000</button>
        <button onClick={clearCompleted}>Clear Completed</button>
      </section>

      <section className="filters">
        <button
          className={filter === 'all' ? 'active' : ''}
          onClick={() => setFilter('all')}
        >
          All
        </button>
        <button
          className={filter === 'active' ? 'active' : ''}
          onClick={() => setFilter('active')}
        >
          Active
        </button>
        <button
          className={filter === 'completed' ? 'active' : ''}
          onClick={() => setFilter('completed')}
        >
          Completed
        </button>
      </section>

      <ul className="todo-list">
        {filteredTodos.map(todo => (
          <li key={todo.id} className={todo.completed ? 'completed' : ''}>
            <input
              type="checkbox"
              checked={todo.completed}
              onChange={() => toggleTodo(todo.id)}
            />
            <span className="todo-text">{todo.text}</span>
            <button className="delete" onClick={() => deleteTodo(todo.id)}>×</button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;

