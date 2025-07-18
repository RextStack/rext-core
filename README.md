# Rext Core

The core library required to run Rext, the fullstack, batteries included framework.

# Contribution Instructions

There is a pre-commit hook in /hooks, please copy it to .git and make it executable, like so:
```bash
cp hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

This ensures `cargo fmt`, `cargo clippy`, and `cargo test` run on every pre-commit. Overkill? Possibly.