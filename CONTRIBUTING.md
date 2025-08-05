# Contributing to Rust R2

We welcome contributions to the Rust R2 project! This document provides guidelines for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/rust-r2.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test your changes thoroughly
6. Commit with clear messages
7. Push to your fork
8. Create a Pull Request

## Development Setup

### Prerequisites
- Rust 1.70+
- Git
- GPG (for testing encryption features)

### Building
```bash
cargo build
cargo test
cargo run --bin rust-r2-cli -- --help
cargo run --bin rust-r2-gui
```

## Code Style

### Rust Guidelines
- Follow standard Rust naming conventions
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add documentation comments for public APIs
- Write unit tests for new functionality

### Commit Messages
- Use present tense ("Add feature" not "Added feature")
- Keep first line under 50 characters
- Reference issues and pull requests when applicable
- Example: "Add folder upload support (#123)"

## Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Test Coverage
- Write unit tests for new functions
- Add integration tests for new features
- Test error cases and edge conditions
- Verify both CLI and GUI functionality

## Pull Request Process

1. **Before Submitting**
   - Update documentation if needed
   - Add tests for new functionality
   - Ensure all tests pass
   - Run `cargo fmt` and `cargo clippy`

2. **PR Description**
   - Clearly describe the changes
   - Link related issues
   - Include screenshots for UI changes
   - List breaking changes if any

3. **Review Process**
   - Address reviewer feedback promptly
   - Keep PR focused on single feature/fix
   - Rebase on main if needed
   - Squash commits if requested

## Feature Requests

### Proposing Features
1. Check existing issues first
2. Open new issue with "Feature Request" label
3. Provide clear use case
4. Describe expected behavior
5. Include mockups for UI changes

### Implementing Features
1. Discuss in issue before implementing
2. Follow architecture patterns
3. Add configuration options if needed
4. Update documentation
5. Add examples

## Bug Reports

### Reporting Bugs
Include:
- Rust version (`rustc --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Error messages or logs
- Configuration used (redact secrets)

### Fixing Bugs
1. Reproduce the issue locally
2. Write failing test
3. Implement fix
4. Verify test passes
5. Check for regressions

## Documentation

### Areas Needing Documentation
- New features
- Configuration options
- API changes
- Examples and tutorials
- Troubleshooting guides

### Documentation Standards
- Use clear, concise language
- Include code examples
- Add screenshots for GUI features
- Keep README focused
- Detailed docs go in `/docs`

## Architecture Guidelines

### Code Organization
```
src/
├── main.rs           # CLI entry point
├── gui/              # GUI application
│   ├── main.rs      # GUI entry point
│   ├── app.rs       # Main application state
│   └── tabs/        # Tab implementations
├── r2_client.rs     # R2 API client
├── crypto.rs        # PGP encryption/decryption
├── config.rs        # Configuration handling
└── lib.rs           # Shared library code
```

### Design Principles
- Separation of concerns
- Thread-safe state management
- Error handling with context
- Async operations for I/O
- Progressive enhancement

## Community

### Communication
- Be respectful and inclusive
- Help others when possible
- Ask questions in issues
- Share knowledge and experiences

### Code of Conduct
- Foster welcoming environment
- Be patient with new contributors
- Provide constructive feedback
- Report inappropriate behavior

## Release Process

### Version Numbering
- Follow Semantic Versioning (SemVer)
- MAJOR.MINOR.PATCH format
- Update Cargo.toml version

### Release Checklist
1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Run full test suite
4. Build release binaries
5. Create git tag
6. Push tag to trigger CI
7. Verify GitHub release

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to open an issue for:
- Clarification on guidelines
- Help with development setup
- Discussion of implementation approach
- General questions about the project

Thank you for contributing to Rust R2!