# Contributing to bevy-plugin-builder

Thank you for your interest in contributing to bevy-plugin-builder. This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork:
```bash
git clone https://github.com/yourusername/bevy-plugin-builder.git
```
3. Create a new branch:
```bash
git checkout -b feature/your-feature-name
```

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/noahsabaj/bevy-plugin-builder.git
cd bevy-plugin-builder

# Build the project
cargo build --all-features

# Run tests
cargo test --all-features

# Run examples
cargo run --example basic_plugin
cargo run --example complex_plugin
cargo run --example migration_guide

# Run clippy for linting
cargo clippy --all-features -- -D warnings

# Check formatting
cargo fmt --check
```

## Coding Standards

### General Rules
- Follow Rust standard naming conventions
- Use `cargo fmt` before committing
- Ensure no clippy warnings with `cargo clippy --all-features -- -D warnings`
- Add documentation for all public APIs
- Write tests for new functionality
- No emojis in code or documentation
- Use factual, utilitarian language in documentation

### Macro Development Guidelines

When modifying or extending the `define_plugin!` macro:

1. **Maintain backward compatibility** - Existing syntax must continue to work
2. **Follow the existing pattern** - New options should integrate seamlessly
3. **Update macro documentation** - Document new configuration options
4. **Add integration tests** - Test new functionality in `tests/integration.rs`
5. **Consider compilation errors** - Provide clear error messages for invalid syntax

### Architecture Guidelines

#### Macro Structure
The macro system consists of three components:
- `define_plugin!` - Main entry point (src/macros.rs:72-87)
- `define_plugin_internal!` - Recursive parser for build() method
- `define_plugin_finish!` - Handler for finish() method

#### Adding New Configuration Options
1. Add new match arm in `define_plugin_internal!` macro
2. Implement corresponding Bevy registration logic
3. Add documentation to macro comments
4. Create test case in `tests/integration.rs`
5. Update examples if relevant

### Commit Messages
- Use present tense ("Add feature" not "Added feature")
- Keep first line under 72 characters
- Reference issues when applicable (#123)
- Be descriptive but concise

## Testing

### Running Tests
```bash
# Run all tests
cargo test --all-features

# Run specific test
cargo test test_basic_plugin

# Run with verbose output
cargo test --all-features -- --nocapture

# Test macro expansion (requires cargo-expand)
cargo expand --example basic_plugin
```

### Writing Tests
- Add integration tests in `tests/integration.rs`
- Test both compilation success and runtime behavior
- Include edge cases and error conditions
- Use descriptive test names

### Testing Macros
When testing macro changes:
1. Verify macro expansion produces correct code
2. Test compilation with various input combinations
3. Ensure generated code follows Bevy patterns
4. Check error messages for invalid syntax

## Documentation

### Documentation Style
- Use clear, factual descriptions
- Avoid promotional language
- Focus on technical accuracy
- Provide practical examples
- No emojis or marketing terms

### Areas Requiring Documentation
- Public macros and their options
- Configuration parameters
- Example usage patterns
- Migration guides for breaking changes

## Pull Request Process

1. Ensure all tests pass: `cargo test --all-features`
2. Run clippy: `cargo clippy --all-features -- -D warnings`
3. Format code: `cargo fmt`
4. Update documentation if needed
5. Add your changes to CHANGELOG.md
6. Submit PR with clear description

### PR Requirements
- Clear description of changes
- Tests for new functionality
- No breaking changes without discussion
- Documentation updates where needed
- Clean commit history (squash if needed)
- Examples demonstrating new features (if applicable)

## Code Review

All submissions require review. We aim to:
- Respond within 48 hours
- Provide constructive feedback
- Merge PRs that meet project standards

## Common Tasks

### Adding Support for New Bevy Features
1. Identify the Bevy API to support
2. Add configuration option to macro
3. Map to appropriate Bevy method calls
4. Test with realistic examples
5. Document the new option

### Improving Error Messages
1. Identify confusing compilation errors
2. Add better error handling in macro
3. Provide actionable error messages
4. Include examples in error text when helpful

### Performance Improvements
1. Profile macro expansion time
2. Optimize recursive macro calls
3. Reduce generated code size
4. Maintain zero runtime overhead

## Release Process

### Automated Publishing with Trusted Publishing

This project uses crates.io's Trusted Publishing feature for secure, automated releases:

1. **Update version in `Cargo.toml`**
2. **Update `CHANGELOG.md`:**
   - Move items from `[Unreleased]` to new version section
   - Add current date (run `date` command to verify)
   - Update comparison links at bottom
3. **Commit changes:**
   ```bash
   git add -A
   git commit -m "Release vX.Y.Z - Brief description"
   ```
4. **Create and push tag:**
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin main
   git push origin vX.Y.Z  # This triggers automatic publishing
   ```
5. **Monitor release:**
   - Check [GitHub Actions](https://github.com/noahsabaj/bevy-plugin-builder/actions)
   - Release workflow will automatically publish to crates.io using OIDC

### Important Notes
- Only version tags (v*) can trigger releases due to environment protection rules
- No API tokens required - uses Trusted Publishing with OIDC authentication
- The `release` environment in GitHub repository settings controls deployment

## Questions?

Feel free to:
- Open an issue for discussion
- Ask questions in PR comments
- Reach out on [Bevy Discord](https://discord.gg/bevy)

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.