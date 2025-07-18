#!/bin/bash

set -e

echo "🚀 Bootstrapping rext-core development environment..."

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if Rust is installed
if ! command_exists cargo; then
    echo "❌ Error: Rust/Cargo is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust/Cargo found"

# Check if git-cliff is installed
if ! command_exists git-cliff; then
    echo "📦 Installing git-cliff..."
    cargo install git-cliff
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install git-cliff"
        exit 1
    fi
    echo "✅ git-cliff installed successfully"
else
    echo "✅ git-cliff found"
fi

# Set up pre-commit hook
if [ -f "hooks/pre-commit" ]; then
    echo "🔗 Setting up pre-commit hook..."

    # Create .git/hooks directory if it doesn't exist
    mkdir -p .git/hooks

    # Copy the pre-commit hook
    cp hooks/pre-commit .git/hooks/pre-commit

    # Make it executable
    chmod +x .git/hooks/pre-commit

    echo "✅ Pre-commit hook installed"
else
    echo "⚠️  Warning: hooks/pre-commit file not found"
fi

echo ""
echo "🎉 Bootstrap complete! You can now:"
echo "   • Run 'cargo build' to build the project"
echo "   • Run 'cargo test' to run tests"
echo "   • Run 'git-cliff -o CLIFF_CHANGELOG.md' to generate a cliff changelog"
echo "   • Make commits and the pre-commit hook will run automatically"
echo ""
echo "📖 See CONTRIBUTING.md for detailed contribution guidelines"