//! TodoMVC Benchmark Runner
//!
//! Runs performance benchmarks comparing Ferric against React TodoMVC implementations.

use wasm_bindgen::prelude::*;
use web_sys::{Performance, Window};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub framework: String,
    pub test: String,
    pub operations: u32,
    pub time_ms: f64,
    pub ops_per_second: f64,
    pub memory_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub name: String,
    pub timestamp: String,
    pub results: Vec<BenchmarkResult>,
    pub summary: BenchmarkSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub ferric_avg_time: f64,
    pub react_avg_time: f64,
    pub ferric_faster_by: f64,
    pub total_tests: usize,
}

/// Get browser performance API
fn get_performance() -> Performance {
    web_sys::window()
        .expect("should have window")
        .performance()
        .expect("should have performance")
}

/// Measure execution time of a function
pub fn measure_time<F, R>(f: F) -> (R, f64)
where
    F: FnOnce() -> R,
{
    let perf = get_performance();
    let start = perf.now();
    let result = f();
    let end = perf.now();
    (result, end - start)
}

/// Estimate memory usage (approximate)
pub fn estimate_memory_mb() -> f64 {
    // This is an approximation - actual memory profiling requires browser DevTools
    let window = web_sys::window().expect("should have window");

    // Try to get memory info if available (Chrome only)
    if let Ok(memory) = js_sys::Reflect::get(
        &js_sys::Reflect::get(&window, &"performance".into()).unwrap(),
        &"memory".into()
    ) {
        if let Ok(used) = js_sys::Reflect::get(&memory, &"usedJSHeapSize".into()) {
            if let Some(used_bytes) = used.as_f64() {
                return used_bytes / 1024.0 / 1024.0;
            }
        }
    }

    0.0 // Not available
}

#[wasm_bindgen]
pub struct BenchmarkRunner {
    results: Vec<BenchmarkResult>,
}

#[wasm_bindgen]
impl BenchmarkRunner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            results: Vec::new(),
        }
    }

    /// Run a benchmark test
    #[wasm_bindgen(js_name = runTest)]
    pub fn run_test(&mut self, framework: String, test: String, operations: u32, time_ms: f64) {
        let ops_per_second = if time_ms > 0.0 {
            (operations as f64 / time_ms) * 1000.0
        } else {
            0.0
        };

        let memory_mb = estimate_memory_mb();

        let result = BenchmarkResult {
            framework,
            test,
            operations,
            time_ms,
            ops_per_second,
            memory_mb,
        };

        self.results.push(result);
    }

    /// Get all results as JSON
    #[wasm_bindgen(js_name = getResults)]
    pub fn get_results(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.results).unwrap()
    }

    /// Generate benchmark suite with summary
    #[wasm_bindgen(js_name = generateSuite)]
    pub fn generate_suite(&self, name: String) -> JsValue {
        let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap();

        // Calculate summary
        let ferric_results: Vec<_> = self.results.iter()
            .filter(|r| r.framework == "Ferric")
            .collect();

        let react_results: Vec<_> = self.results.iter()
            .filter(|r| r.framework == "React")
            .collect();

        let ferric_avg_time = if !ferric_results.is_empty() {
            ferric_results.iter().map(|r| r.time_ms).sum::<f64>() / ferric_results.len() as f64
        } else {
            0.0
        };

        let react_avg_time = if !react_results.is_empty() {
            react_results.iter().map(|r| r.time_ms).sum::<f64>() / react_results.len() as f64
        } else {
            0.0
        };

        let ferric_faster_by = if react_avg_time > 0.0 {
            ((react_avg_time - ferric_avg_time) / react_avg_time) * 100.0
        } else {
            0.0
        };

        let summary = BenchmarkSummary {
            ferric_avg_time,
            react_avg_time,
            ferric_faster_by,
            total_tests: self.results.len(),
        };

        let suite = BenchmarkSuite {
            name,
            timestamp,
            results: self.results.clone(),
            summary,
        };

        serde_wasm_bindgen::to_value(&suite).unwrap()
    }

    /// Clear all results
    #[wasm_bindgen(js_name = clearResults)]
    pub fn clear_results(&mut self) {
        self.results.clear();
    }
}

/// Benchmark helper for TodoMVC operations
#[wasm_bindgen]
pub struct TodoBenchmark;

#[wasm_bindgen]
impl TodoBenchmark {
    /// Benchmark adding N todos
    #[wasm_bindgen(js_name = benchmarkAdd)]
    pub fn benchmark_add(count: u32) -> f64 {
        let (_, time) = measure_time(|| {
            for i in 0..count {
                // Simulate todo creation
                let _ = format!("Todo {}", i);
            }
        });
        time
    }

    /// Benchmark toggling N todos
    #[wasm_bindgen(js_name = benchmarkToggle)]
    pub fn benchmark_toggle(count: u32) -> f64 {
        let mut todos = vec![false; count as usize];

        let (_, time) = measure_time(|| {
            for i in 0..count {
                todos[i as usize] = !todos[i as usize];
            }
        });
        time
    }

    /// Benchmark deleting N todos
    #[wasm_bindgen(js_name = benchmarkDelete)]
    pub fn benchmark_delete(count: u32) -> f64 {
        let mut todos: Vec<String> = (0..count)
            .map(|i| format!("Todo {}", i))
            .collect();

        let (_, time) = measure_time(|| {
            while !todos.is_empty() {
                todos.pop();
            }
        });
        time
    }

    /// Benchmark filtering N todos
    #[wasm_bindgen(js_name = benchmarkFilter)]
    pub fn benchmark_filter(count: u32) -> f64 {
        let todos: Vec<(String, bool)> = (0..count)
            .map(|i| (format!("Todo {}", i), i % 2 == 0))
            .collect();

        let (_, time) = measure_time(|| {
            let _active: Vec<_> = todos.iter()
                .filter(|(_, completed)| !completed)
                .collect();
            let _completed: Vec<_> = todos.iter()
                .filter(|(_, completed)| *completed)
                .collect();
        });
        time
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Benchmark runner initialized".into());
}

