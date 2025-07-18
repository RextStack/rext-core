# Rext Core

The core library required to run Rext, the fullstack, batteries included framework.

# Contribution Instructions

There is a pre-commit hook in /hooks, please copy it to .git like so:
```bash
cp rext-core/hooks/pre-commit rext-core/.git/hooks/pre-commit
```

This ensures `cargo fmt`, `cargo clippy`, and `cargo test` run on every pre-commit. Overkill? Possibly.