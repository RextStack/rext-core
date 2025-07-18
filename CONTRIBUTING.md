# Contributing to rext-core

Thank you for your interest in contributing to rext-core! This document outlines the guidelines and best practices for contributing to this project.

## Prerequisites

- **Rust**: Latest stable version (minimum Rust 2024 edition support)
- **Git**: For version control
- Basic familiarity with async Rust and web development concepts

## Development Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd rext-core
   ```

2. **Install pre-commit hooks**
   ```bash
   cp hooks/pre-commit .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

   This ensures `cargo fmt`, `cargo clippy`, and `cargo test` run automatically before each commit.

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

## Code Style and Quality

### Formatting
- Use `cargo fmt` to format your code
- The rext-core.workspace file will run this on save, if you're working in Visual Studio Code (or a fork of it)
- The pre-commit hook will automatically run this

### Linting
- Fix all `cargo clippy` warnings
- Aim for clippy score of zero warnings

### Documentation
- Document all public APIs with doc comments (`///`)
- Include examples in documentation when helpful
- Run `cargo doc --open` to preview documentation

## Testing Guidelines

### Test Requirements
- All new functionality must include tests
- Maintain or improve current test coverage
- Tests should be deterministic and not rely on external services

### Test Types
- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test component interactions
- **Documentation tests**: Ensure code examples in docs work

### Running Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Follow the code style guidelines
   - Add tests for new functionality
   - Update documentation as needed

3. **Commit your changes**
   - Use clear, descriptive commit messages
   - Follow conventional commit format when possible
   - The pre-commit hook will run automatically

4. **Push and create PR**
   - Push your branch to your fork
   - Create a pull request with a clear description
   - Reference any related issues

### PR Requirements
- [ ] All tests pass (`cargo test`)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated if needed
- [ ] New functionality includes tests

## Architecture Guidelines

### Error Handling
- Use the `RextCoreError` enum for all library errors
- Follow the established pattern for error context
- Don't use `unwrap()` or `expect()` in library code
- Errors should bubble up to the calling process

### Async Code
- Prefer async/await over raw futures
- Use Tokio's async primitives consistently
- Follow async best practices (avoid blocking in async functions)

### API Design
- Design APIs that are easy to use correctly and hard to use incorrectly
- Maintain backward compatibility when possible

## Contribution Areas

We welcome contributions all areas of the rext-core crate, such as:
- **Core functionality**: Routing, middleware
- **Documentation**: API docs, guides, examples
- **Testing**: Expanding test coverage, improving test quality
- **Performance**: Optimizations and benchmarks
- **Error handling**: Better error messages and debugging support

## Getting Help

- Check existing issues and discussions
- Feel free to open an issue for questions or clarification
- Be respectful and constructive in all interactions

## Use of AI

AI is a useful programming tool, but shouldn't write your entire PR for you.
- Use AI when appropriate and carefully review all changes it makes
- Avoid having it write large swaths of code at a time, as this makes code review challenging and makes everyone more unfamiliar with the code base
- Do use AI for code reviews, suggesting improvements, optimizations, better/more test coverage, or documentation
- AI tends to write VERY verbose Rust Docs; please review all doc comments to make sure they are succinct and necessary

## License

By contributing to rext-core, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to rext-core! 🦀
