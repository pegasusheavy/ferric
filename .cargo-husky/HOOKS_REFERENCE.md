# Git Hooks Quick Reference

## 🔍 Pre-commit Hook
**Runs:** Before each commit
**Checks:** Code formatting and linting

**Fix issues:**
```bash
cargo fmt --all
cargo clippy --all-targets --all-features --workspace -- -D warnings
```

---

## 🧪 Pre-push Hook
**Runs:** Before pushing to remote
**Checks:** All tests pass

**Fix issues:**
```bash
cargo test --workspace --all-features
```

---

## 📝 Commit-msg Hook
**Runs:** When writing commit message
**Checks:** Conventional Commits format

**Format:**
```
<type>[optional scope]: <description>
```

**Valid types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

**Examples:**
```bash
git commit -m "feat: add new feature"
git commit -m "fix(router): resolve bug"
git commit -m "docs: update README"
```

---

## ⚠️ Skip Hooks (Emergency Only)
```bash
git commit --no-verify -m "emergency fix"
git push --no-verify
```

---

## 🔧 Reinstall Hooks
```bash
.cargo-husky/install-hooks.sh
```

---

## 📖 Full Documentation
See [README.md](.cargo-husky/README.md) for complete details.

