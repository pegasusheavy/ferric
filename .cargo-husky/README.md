# Git Hooks Setup

This directory contains Git hooks managed by cargo-husky for the Ferric project.

## Overview

The following Git hooks are configured:

### 🔍 Pre-commit Hook
Runs before each commit to ensure code quality:
- **Formatting Check**: Verifies code is formatted with `cargo fmt`
- **Linting**: Runs `cargo clippy` with warnings denied

**What to do if it fails:**
```bash
# Fix formatting issues
cargo fmt --all

# Fix clippy warnings
cargo clippy --all-targets --all-features --workspace -- -D warnings
```

### 🧪 Pre-push Hook
Runs before pushing to ensure all tests pass:
- **Tests**: Runs the full test suite with `cargo test --workspace --all-features`

**What to do if it fails:**
```bash
# Run tests locally to see failures
cargo test --workspace --all-features

# Fix any failing tests before pushing
```

### 📝 Commit-msg Hook
Validates commit message format (Conventional Commits):
- Enforces structured commit messages for better changelog generation

**Valid commit message format:**
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that don't affect code meaning (formatting, etc.)
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or correcting tests
- `build`: Changes to build system or dependencies
- `ci`: Changes to CI configuration
- `chore`: Other changes that don't modify src or test files
- `revert`: Reverts a previous commit

**Examples:**
```bash
git commit -m "feat: add reactive signals system"
git commit -m "fix(router): resolve navigation race condition"
git commit -m "docs: update README with new examples"
git commit -m "refactor(core): simplify dependency injection"
git commit -m "test(forms): add validation tests"
```

## Installation

### Automatic Installation
The hooks are automatically installed when you build the project:
```bash
cargo build
```

### Manual Installation
If you need to reinstall the hooks:
```bash
.cargo-husky/install-hooks.sh
```

Or manually:
```bash
cp .cargo-husky/hooks/* .git/hooks/
chmod +x .git/hooks/pre-commit .git/hooks/pre-push .git/hooks/commit-msg
```

## Skipping Hooks

Sometimes you may need to skip hooks (use sparingly!):

```bash
# Skip pre-commit hooks
git commit --no-verify -m "fix: emergency hotfix"

# Skip pre-push hooks
git push --no-verify
```

## Modifying Hooks

The hook scripts are located in `.cargo-husky/hooks/`. After modifying them:

1. Edit the hook script in `.cargo-husky/hooks/`
2. Run the installation script: `.cargo-husky/install-hooks.sh`
3. Test the hook to ensure it works as expected

## Troubleshooting

### Hooks not running
- Ensure hooks are executable: `chmod +x .git/hooks/*`
- Reinstall hooks: `.cargo-husky/install-hooks.sh`

### False positives
If a hook is incorrectly failing:
1. Run the command manually to debug
2. Fix the underlying issue (don't skip the hook!)
3. Consider updating the hook script if there's a legitimate issue

### Performance issues
If hooks are too slow:
- Consider caching cargo builds
- Adjust hook scripts to check only modified files
- Use `--no-verify` sparingly for emergency situations

## CI/CD

These same checks run in CI/CD pipelines. Hooks help catch issues early before pushing.

