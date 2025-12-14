# Testing and Coverage Guide

Comprehensive guide to testing and code coverage in the Ferric framework.

## Quick Start

### Run All Tests

```bash
cargo test --lib
```

### Run Tests with Coverage

```bash
bash scripts/run-coverage.sh
```

Or directly:

```bash
cargo tarpaulin --lib --out Html --output-dir coverage
```

### View Coverage Report

```bash
firefox coverage/index.html
```

## Current Coverage Status

**Baseline Coverage: ~31%**

**Goal: 90%+**

### Coverage by Module

| Module | Coverage | Status |
|--------|----------|--------|
| ferric-core | ~31% | 🟡 Needs improvement |
| ferric-forms | ~0% | 🔴 Critical |
| ferric-http | ~0% | 🔴 Critical |
| ferric-idb | ~0% | 🔴 Critical |
| ferric-ssr | ~0% | 🔴 Critical |
| ferric-markdown | ~0% | 🔴 Critical |

## Testing Strategy

### Unit Tests

Test individual functions and methods in isolation.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let s = signal(42);
        assert_eq!(s.get(), 42);
    }
}
```

### Integration Tests

Test module interactions in `tests/` directory.

```rust
// tests/integration_tests.rs
use ferric_core::prelude::*;

#[test]
fn test_di_with_signals() {
    // Test dependency injection with reactive state
}
```

### Property-Based Tests

Use `proptest` or `quickcheck` for property testing.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_signal_always_returns_last_set(value in any::<i32>()) {
        let s = signal(0);
        s.set(value);
        assert_eq!(s.get(), value);
    }
}
```

## Writing Tests

### Test Organization

```
crate/
├── src/
│   ├── module.rs
│   └── module/
│       ├── mod.rs
│       ├── submodule.rs  # impl code
│       └── tests.rs      # unit tests (or inline #[cfg(test)])
└── tests/
    └── integration.rs    # integration tests
```

### Test Naming

```rust
#[test]
fn test_<what>_<condition>_<expected>() {
    // Examples:
    // test_signal_update_increments_value
    // test_router_navigate_invalid_route_returns_error
    // test_di_resolve_unregistered_service_returns_none
}
```

### AAA Pattern

- **Arrange**: Set up test data
- **Act**: Execute the code being tested
- **Assert**: Verify the results

```rust
#[test]
fn test_example() {
    // Arrange
    let service = MyService::new();
    let input = "test";

    // Act
    let result = service.process(input);

    // Assert
    assert_eq!(result, "TEST");
}
```

## Coverage Tools

### Tarpaulin (Recommended)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

**Pros:**
- Fast
- Accurate
- HTML reports
- CI-friendly

**Cons:**
- Linux only

### LLVM Coverage (Alternative)

```bash
# Install llvm-tools
rustup component add llvm-tools-preview

# Set environment variables
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"

# Run tests
cargo test

# Generate report
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

## CI/CD Integration

### GitHub Actions

Located at `.github/workflows/coverage.yml`:

- Runs on push to main and PRs
- Uploads to Codecov
- Posts coverage comment on PRs
- Generates HTML reports

### Coverage Badge

Add to `README.md`:

```markdown
[![codecov](https://codecov.io/gh/username/ferric/branch/main/graph/badge.svg)](https://codecov.io/gh/username/ferric)
```

## Improving Coverage

### 1. Identify Uncovered Code

```bash
cargo tarpaulin --out Html
firefox coverage/index.html
```

Look for red lines in the HTML report.

### 2. Write Tests for Uncovered Lines

Focus on:
- Error paths
- Edge cases
- Conditional branches
- Match arms

### 3. Use Coverage-Guided Testing

```bash
# Run coverage iteratively
bash scripts/run-coverage.sh

# Identify lowest coverage modules
grep "0/" coverage_run.log

# Write tests for those modules
# Repeat until 90%+ coverage
```

## Testing Best Practices

### 1. Test Public API

```rust
// ✅ Good: Test public interface
#[test]
fn test_public_method() {
    let service = PublicService::new();
    assert_eq!(service.do_something(), expected);
}

// ❌ Bad: Test private implementation
#[test]
fn test_private_helper() {
    // Don't test private methods directly
}
```

### 2. Mock External Dependencies

```rust
#[cfg(test)]
use mockall::automock;

#[automock]
trait HttpClient {
    fn get(&self, url: &str) -> Result<String>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockHttpClient::new();
    mock.expect_get()
        .returning(|_| Ok("response".to_string()));

    // Test code using mock
}
```

### 3. Use Test Fixtures

```rust
struct TestFixture {
    injector: Injector,
    service: Rc<TestService>,
}

impl TestFixture {
    fn new() -> Self {
        let injector = Injector::root();
        // Setup
        Self { injector, service }
    }
}

#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new();
    // Use fixture.injector, fixture.service
}
```

### 4. Test Error Conditions

```rust
#[test]
fn test_error_handling() {
    let result = function_that_can_fail(invalid_input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ExpectedError);
}

#[test]
#[should_panic(expected = "panic message")]
fn test_panic_condition() {
    function_that_panics();
}
```

### 5. Async Testing

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
async fn test_async_function() {
    let result = async_function().await;
    assert_eq!(result, expected);
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert_eq!(result, expected);
}
```

## Coverage Goals

### Minimal Acceptable Coverage

- **Core modules**: 90%+
- **Utility modules**: 85%+
- **Test utilities**: 70%+
- **Examples**: Not measured

### Priority Modules for 90% Coverage

1. `ferric-core/src/reactive/` - Signals, computed, effects
2. `ferric-core/src/di/` - Dependency injection
3. `ferric-core/src/router/` - Routing
4. `ferric-forms/src/` - Form controls and validation
5. `ferric-http/src/` - HTTP client

## Continuous Improvement

### Weekly Coverage Review

1. Run coverage report
2. Identify modules below target
3. Write tests for uncovered code
4. Review PRs for test coverage
5. Update coverage badge

### PR Requirements

- All new code must have tests
- Coverage should not decrease
- Aim for 90%+ on changed files

## Troubleshooting

### Tarpaulin Fails to Build

```bash
# Update tarpaulin
cargo install cargo-tarpaulin --locked --force

# Or use LLVM coverage instead
cargo llvm-cov --html
```

### Tests Pass Locally but Fail in CI

- Check for timing issues (use `tokio::time::pause()`)
- Verify test isolation (no shared mutable state)
- Check for filesystem dependencies

### Coverage Seems Wrong

- Ensure tests actually run the code paths
- Check for `#[cfg(test)]` accidentally excluding code
- Verify tarpaulin is analyzing correct files

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Codecov Documentation](https://docs.codecov.com/)
- [Property-Based Testing](https://github.com/proptest-rs/proptest)

## License

MIT

