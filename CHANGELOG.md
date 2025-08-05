# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-07-19

### Changed

- Refactored tests: moved integration tests from lib.rs to dedicated tests/integration_tests.rs file
- Updated Cargo.toml
- Renamed the License file
- Updated README
- Removed the 'placeholder' logic and tests

### Fixed

- Updated the homepage URL in Cargo.toml, was missing the https://www
- Fixed the output directory for sea-orm-cli command to be backend/entity/models (new Rext architecture)

### Added

- Badges to the README
- Added check_for_rext_app to check if a rext project exists in the current directory
- Added scaffold_rext_app to scaffold a new rext project
- Added destroy_rext_app to destroy a rext project
- Added DirectoryCreation, FileWrite, AppAlreadyExists, CurrentDir, DirectoryRead, FileRemoval, DirectoryRemoval, and SafetyCheck errors
- Added SeaOrmCliGenerateEntities error
- Added TYPES_TO_WRAP and ENTITIES_DIR constants
- Added ignore-tables flag to sea-orm-cli command to ignore jobs and workers tables

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