---
description: Comprehensive Rust code review for ownership, lifetimes, error handling, unsafe usage, and idiomatic patterns. Invokes the rust-reviewer agent.
---

# Rust Code Review

This command invokes the **rust-reviewer** agent for comprehensive Rust-specific code review.

## What This Command Does
1. **Verify Automated Checks**: Run `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and `cargo test` — stop if any fail
2. **Identify Rust Changes**: Find modified `.rs` files via `git diff HEAD~1` (or `git diff main...HEAD` for PRs)
3. **Run Security Audit**: Execute `cargo audit` if available
4. **Security Scan**: Check for unsafe usage, command injection, hardcoded secrets
5. **Ownership Review**: Analyze unnecessary clones, lifetime issues, borrowing patterns
6. **Generate Report**: Categorize issues by severity

## When to Use
Use `/rust-review` when:
- After writing or modifying Rust code
- Before committing Rust changes
- Reviewing pull requests with Rust code
- Onboarding to a new Rust codebase
- Learning idiomatic Rust patterns

## Review Categories
### CRITICAL (Must Fix)
- Unchecked `unwrap()`/`expect()` in production code paths
- `unsafe` without `// SAFETY:` comment documenting invariants
- SQL injection via string interpolation in queries
- Command injection via unvalidated input in `std::process::Command`
- Hardcoded credentials

### HIGH (Should Fix)
- Unnecessary `.clone()` to satisfy borrow checker
- `String` parameter where `&str` or `impl AsRef<str>` suffices
- Blocking in async context (`std::thread::sleep`, `std::fs`)
- Missing `Send`/`Sync` bounds on shared types
- Wildcard `_ =>` match on business-critical enums
- Large functions (>50 lines)

## Automated Checks Run
```bash
# Build gate (must pass before review)
cargo check

# Lints and suggestions
cargo clippy -- -D warnings

# Formatting
cargo fmt --check

# Tests
cargo test

# Security audit (if available)
if command -v cargo-audit >/dev/null; then cargo audit; else echo "cargo-audit not installed"; fi
```

## Example Usage
````text
User: /rust-review

Agent:
# Rust Code Review Report
...
```
