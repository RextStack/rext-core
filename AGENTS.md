# AGENTS.md - Project Context

## Project Overview

**rext-core** is the core library that powers Rext, a fullstack, batteries-included Rust framework for developing web applications. This project aims to handle the fundamental requirements that nearly all web applications share, including routing, API documentation, and front-end capabilities.

## Current Status

- **Development Stage**: Early development (0% complete)
- **Version**: 0.1.0
- **Rust Edition**: 2024

## Architecture

### Architecture Goal

- rext-code aims to act as a microkernel so additional rext-* crates can act as plugin modules.

### Core Technologies
- **Web Framework**: Axum (0.8.4) - Modern async web framework
- **Async Runtime**: Tokio (1.46.1) - Full-featured async runtime
- **Error Handling**: thiserror (2.0.12) - Structured error handling

### Key Components

1. **Error Handling**
   - `RextCoreError` enum with structured error types
   - Proper error propagation and context

### Project Structure
```
rext-core/
├── src/lib.rs          # Main library code
├── Cargo.toml          # Dependencies and metadata
├── hooks/pre-commit    # Git pre-commit hooks
└── tests/              # Integration tests (in lib.rs currently)
```

## Development Notes

- The project uses comprehensive testing with reqwest for HTTP client testing
- Pre-commit hooks ensure code quality (fmt, clippy, test)
- Server supports both blocking and non-blocking operation modes
- Error handling follows Rust best practices with custom error types

## Goals

Rext aims to be a batteries-included framework, meaning developers should be able to build complete web applications with minimal external dependencies and configuration.

Visit [Rext Stack](https://rextstack.org) for more information.

## For AI Agents

When working on this project:
- Follow Rust 2024 edition conventions
- Maintain async-first design patterns
- Use structured error handling with `RextCoreError`
- Ensure all code is properly tested
- Never add new features that were not requested
- When adding a new feature, reason if it should be core or a separate plugin
- Follow the established 'microkernel' architecutre for core