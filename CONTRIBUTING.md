# Contributing to Ferric

Thank you for your interest in contributing to Ferric! This guide will help you get started.

## Development Setup

### Prerequisites
- Rust 1.70+ with `rustfmt` and `clippy` installed
- Git
- wasm-pack (for WebAssembly builds)

### Initial Setup
1. Clone the repository
2. Install dependencies: `cargo build`
3. Git hooks will be automatically set up

## Git Hooks

This project uses Git hooks to maintain code quality. They are managed by cargo-husky.

### Pre-commit
- Checks code formatting (`cargo fmt --check`)
- Runs linter (`cargo clippy`)

### Pre-push
- Runs the full test suite

### Commit Message Format
We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions or modifications
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Maintenance tasks

**Examples:**
```bash
feat: add signal-based reactivity system
fix(router): resolve race condition in navigation
docs: update component API documentation
refactor(di): simplify injector hierarchy
```

## Development Workflow

### Before Committing
```bash
# Format code
cargo fmt --all

# Check for linting issues
cargo clippy --all-targets --all-features --workspace

# Run tests
cargo test --workspace
```

### Making Changes
1. Create a feature branch: `git checkout -b feat/my-feature`
2. Make your changes
3. Write/update tests
4. Commit with conventional commit message
5. Push and create a pull request

### Running Tests
```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# WASM build
wasm-pack build --target web ferric-core
```

## Code Style

- Follow Rust standard naming conventions
- Use `rustfmt` for formatting (runs automatically in pre-commit)
- Address all `clippy` warnings
- Write documentation for public APIs
- Add tests for new functionality

## Pull Request Guidelines

1. **Title**: Use conventional commit format
2. **Description**: Clearly describe what and why
3. **Tests**: Include tests for new features/fixes
4. **Documentation**: Update docs if needed
5. **Clean history**: Rebase if needed before merging

## Getting Help

- Check the [README](README.md) for project overview
- Review [hook documentation](.cargo-husky/README.md)
- Open an issue for bugs or feature requests

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

