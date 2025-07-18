#!/bin/bash

# Setup script for rext-core development environment

set -e

echo "🔧 Setting up rext-core development environment..."

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit..."

    # Try different installation methods
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v brew &> /dev/null; then
        brew install pre-commit
    elif command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y python3-pip
        pip3 install pre-commit
    else
        echo "❌ Could not install pre-commit. Please install it manually:"
        echo "   pip install pre-commit"
        echo "   or visit: https://pre-commit.com/#installation"
        exit 1
    fi
fi

echo "✅ pre-commit is available"

# Install pre-commit hooks
echo "🪝 Installing pre-commit hooks..."
pre-commit install

echo "🧪 Running pre-commit on all files to test setup..."
pre-commit run --all-files || {
    echo "⚠️  Some pre-commit checks failed. This is normal on first run."
    echo "   Run 'cargo fmt' to fix formatting issues."
}

echo "🚀 Development environment setup complete!"
echo ""
echo "📋 What's configured:"
echo "   • Format on save for Rust files (cargo fmt)"
echo "   • Pre-commit hooks that run:"
echo "     - cargo fmt (format code)"
echo "     - cargo check (check compilation)"
echo "     - cargo test (run tests)"
echo "     - cargo build (build project)"
echo "     - Basic file checks (trailing whitespace, etc.)"
echo ""
echo "💡 To use the workspace file in Cursor/VS Code:"
echo "   File → Open Workspace from File → select 'rext-core.code-workspace'"
