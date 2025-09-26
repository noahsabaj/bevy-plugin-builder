# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.6] - 2025-09-26

### Added
- Integration tests covering nested plugin composition and sub-state initialization to guard macro arms
- `trybuild` UI test framework with compile-fail tests for unknown plugin keys
- Comprehensive test coverage for compile-time error messages

### Fixed
- Dead code warning for unused enum variant in integration tests

## [0.1.5] - 2025-09-24

### Fixed
- CI workflow: Added clippy and rustfmt components to rust-toolchain setup
- All clippy warnings and unused field errors across examples and tests
- Removed unused struct fields to achieve zero warnings
- Converted empty structs to unit structs for cleaner code
- Fixed DiagnosticsPlugin::default() clippy warning

## [0.1.4] - 2025-09-24

### Added
- Configured Trusted Publishing for automated crates.io releases via GitHub Actions
- GitHub environment protection rules for release workflow
- Improved CI/CD documentation

### Fixed
- GitHub Actions workflow YAML syntax errors
- CI workflow configuration (removed non-existent Bevy 0.17 tests)
- Improved clippy configuration with --all-targets --all-features

### Changed
- Release workflow now uses OIDC authentication instead of API tokens
- CI workflow simplified and optimized

## [0.1.3] - 2025-09-24

### Added
- CONTRIBUTING.md with contribution guidelines
- CHANGELOG.md for tracking project changes
- Documentation language guidelines in CLAUDE.md

### Changed
- Documentation updated to use utilitarian, factual language
- Removed all emojis from codebase
- Removed promotional language and marketing claims
- Package description simplified in Cargo.toml

## [0.1.2] - 2025-09-21

### Fixed
- Cargo.lock synchronization

## [0.1.1] - 2025-09-18

### Changed
- Updated README formatting and clarity
- Improved code formatting throughout project

### Fixed
- README content improvements

## [0.1.0] - 2025-09-18

### Added
- Initial release of bevy-plugin-builder
- `define_plugin!` macro for declarative plugin definition
- Support for all Bevy plugin registration patterns:
  - Resources initialization
  - Events registration
  - Plugin composition
  - State management
  - Sub-states
  - Reflection types
  - System scheduling (Startup, Update, FixedUpdate)
  - State transitions (OnEnter, OnExit)
  - Custom initialization logic
  - Custom finish logic
- Three comprehensive examples:
  - basic_plugin.rs - Simple usage demonstration
  - complex_plugin.rs - Advanced features showcase
  - migration_guide.rs - Traditional vs declarative comparison
- Integration test suite
- GitHub Actions CI/CD pipeline:
  - Automated testing and linting
  - Automated crates.io publishing on release
- Dual MIT/Apache-2.0 licensing
- Full Bevy 0.16 compatibility
- Rust 1.87 MSRV (following Bevy's policy)

[Unreleased]: https://github.com/noahsabaj/bevy-plugin-builder/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/noahsabaj/bevy-plugin-builder/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/noahsabaj/bevy-plugin-builder/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/noahsabaj/bevy-plugin-builder/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/noahsabaj/bevy-plugin-builder/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/noahsabaj/bevy-plugin-builder/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/noahsabaj/bevy-plugin-builder/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/noahsabaj/bevy-plugin-builder/releases/tag/v0.1.0
