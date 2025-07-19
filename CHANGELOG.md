# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-07-18

### Added

- CHANGELOG.md file
- README.md file
- CONTRIBUTION.md guide
- AGENTS.md file for helping AI agents (to be used with good intentions)
- lib.rs, core of the rext-core lib
- A pre-commit hook for running cargo commands before commiting changes
- A code-workspace file with some workspace support
- A github workflow, tests and builds commits to main, caches assets
- git-cliff pre-commit hook
- CLIFF_CHANGELOG.md, a git-cliff generated changelog for reference
- A bootstrap script to bootstrap development environment quickly
- Cargo.toml package info
- This initial release is to just jump-start the changelog and releases, nothing decent in it

### Fixed

- Workspace cleanup (removed py pre-commit)

[unreleased]: https://github.com/RextStack/rext-core/releases/tag/v0.1.0