# Development Setup

This document explains how to set up and use the development environment for rext-core.

## Quick Setup

Run the setup script to configure your development environment:

```bash
./setup-dev.sh
```

This script will:
- Install pre-commit hooks
- Configure git hooks for code quality checks
- Test the setup

## Manual Setup

If you prefer to set up manually:

### 1. Install pre-commit

```bash
# Using pip
pip install pre-commit

# Using brew (macOS)
brew install pre-commit

# Using apt (Ubuntu/Debian)
sudo apt-get install python3-pip && pip3 install pre-commit
```

### 2. Install git hooks

```bash
pre-commit install
```

## Workspace Configuration

### Cursor/VS Code Workspace

Open the workspace file for the best development experience:

1. In Cursor/VS Code: `File → Open Workspace from File`
2. Select `rext-core.code-workspace`

This provides:
- **Format on save**: Automatically runs `cargo fmt` when saving Rust files
- **Integrated tasks**: Quick access to build, test, check, and format commands
- **Recommended extensions**: Rust analyzer, debugger, and other useful tools
- **Optimized settings**: Configured for Rust development

### Manual Settings

If not using the workspace file, ensure these settings in `.vscode/settings.json`:

```json
{
    "rust-analyzer.rustfmt.overrideCommand": ["cargo", "fmt", "--"],
    "editor.formatOnSave": true,
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

## Pre-commit Hooks

The following checks run automatically on every commit:

### Code Quality
- **`cargo fmt`**: Formats Rust code according to project style
- **`cargo check`**: Ensures code compiles without errors
- **`cargo test`**: Runs all tests to ensure functionality
- **`cargo build`**: Verifies the project builds successfully

### File Quality
- **Trailing whitespace**: Removes unnecessary trailing spaces
- **End of file**: Ensures files end with newlines
- **Merge conflicts**: Prevents committing unresolved conflicts
- **TOML/YAML**: Validates configuration file syntax

## Development Workflow

### 1. Make Changes
Edit your Rust files. Format-on-save will automatically run `cargo fmt`.

### 2. Test Locally
```bash
cargo test
cargo check
cargo build
```

### 3. Commit Changes
```bash
git add .
git commit -m "Your commit message"
```

Pre-commit hooks will automatically run and must pass before the commit succeeds.

### 4. Manual Pre-commit Run
To run hooks manually on all files:
```bash
pre-commit run --all-files
```

To run hooks on specific files:
```bash
pre-commit run --files src/lib.rs
```

## Available Commands

### Cargo Commands
```bash
cargo build          # Build the project
cargo test           # Run tests
cargo check          # Check for compilation errors
cargo fmt            # Format code
cargo clippy         # Run linting
```

### Pre-commit Commands
```bash
pre-commit run --all-files    # Run all hooks on all files
pre-commit run cargo-fmt      # Run only formatting
pre-commit run cargo-test     # Run only tests
pre-commit install            # Install hooks
pre-commit uninstall          # Remove hooks
```

## Troubleshooting

### Pre-commit Fails
If pre-commit hooks fail:

1. **Formatting issues**: Run `cargo fmt` to fix
2. **Compilation errors**: Fix the errors shown by `cargo check`
3. **Test failures**: Fix failing tests or update test expectations
4. **File issues**: Remove trailing whitespace, fix YAML/TOML syntax

### Skip Hooks (Emergency)
To temporarily skip pre-commit hooks:
```bash
git commit --no-verify -m "Emergency commit"
```

**Note**: Use sparingly and fix issues in the next commit.

### VS Code Extensions

Recommended extensions (auto-suggested when opening workspace):
- **rust-analyzer**: Rust language support
- **vadimcn.vscode-lldb**: Debugging support
- **ms-vscode.vscode-json**: JSON editing
- **redhat.vscode-yaml**: YAML editing

## Configuration Files

- `.vscode/settings.json`: VS Code workspace settings
- `.pre-commit-config.yaml`: Pre-commit hook configuration
- `rext-core.code-workspace`: Complete workspace definition
- `setup-dev.sh`: Automated setup script
