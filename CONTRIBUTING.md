# Contributing to Echomind

Thank you for your interest in contributing to Echomind! This document provides guidelines and instructions for contributing to the project.

## 🤝 Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Basic knowledge of Rust and async programming

### Setting Up Development Environment

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/echomind.git
   cd echomind
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/thepinak503/echomind.git
   ```

4. Install dependencies and build:
   ```bash
   cargo build
   ```

5. Run tests to ensure everything works:
   ```bash
   cargo test
   ```

## 🏗️ Project Structure

```
echomind/
├── src/
│   ├── main.rs       # Entry point
│   ├── lib.rs        # Library exports
│   ├── api.rs        # API client and provider logic
│   ├── cli.rs        # CLI argument parsing
│   ├── config.rs     # Configuration management
│   ├── error.rs      # Error types
│   └── repl.rs       # Interactive REPL mode
├── tests/
│   ├── api_tests.rs     # API tests
│   └── config_tests.rs  # Config tests
├── .github/
│   └── workflows/    # CI/CD workflows
├── Cargo.toml        # Dependencies
└── README.md         # Documentation
```

## 💻 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes

### 2. Make Your Changes

- Write clean, readable code
- Follow Rust conventions and idioms
- Add comments for complex logic
- Update documentation as needed

### 3. Format and Lint

Before committing, ensure code is properly formatted:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

### 4. Commit Changes

Write clear, descriptive commit messages:

```bash
git add .
git commit -m "feat: add support for streaming responses"
```

Commit message format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions/changes
- `chore:` - Maintenance tasks

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title describing the change
- Detailed description of what and why
- Reference any related issues

## 🧪 Testing Guidelines

### Writing Tests

- Add tests for new features
- Ensure existing tests pass
- Aim for good code coverage
- Test edge cases and error conditions

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Test Structure

```rust
#[test]
fn test_feature_name() {
    // Arrange
    let input = setup_test_data();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected_value);
}
```

## 📝 Documentation

### Code Documentation

Use doc comments for public APIs:

```rust
/// Brief description of the function
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Example
///
/// ```
/// let result = my_function("test");
/// ```
pub fn my_function(param: &str) -> String {
    // implementation
}
```

### README Updates

Update README.md when adding:
- New features
- New CLI options
- New API providers
- Usage examples

## 🐛 Bug Reports

### Before Submitting

- Check if the bug is already reported
- Verify it's reproducible
- Check if it's fixed in latest version

### Bug Report Template

```markdown
**Description**
Brief description of the bug

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. With input '...'
3. See error

**Expected Behavior**
What you expected to happen

**Actual Behavior**
What actually happened

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.70]
- Echomind version: [e.g., 0.3.0]

**Additional Context**
Any other relevant information
```

## ✨ Feature Requests

### Before Submitting

- Check if feature is already requested
- Consider if it fits project scope
- Think about implementation approach

### Feature Request Template

```markdown
**Feature Description**
Clear description of the feature

**Use Case**
Why is this feature needed?

**Proposed Solution**
How should it work?

**Alternatives Considered**
Other approaches you've thought about

**Additional Context**
Any other relevant information
```

## 🎯 Areas to Contribute

### High Priority

- Additional API provider support
- Performance improvements
- Better error messages
- More comprehensive tests

### Documentation

- Usage examples
- API provider guides
- Tutorial videos
- Translation to other languages

### Features

- Conversation history persistence
- Output formatting options
- Plugin system
- Clipboard integration

## 🔍 Code Review Process

1. Maintainers review PRs within a few days
2. Address requested changes promptly
3. Keep discussions professional and constructive
4. Once approved, PR will be merged

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 💬 Getting Help

- Open an issue for questions
- Check existing issues and PRs
- Read the documentation thoroughly

## 🙏 Recognition

Contributors are recognized in:
- GitHub contributors page
- Release notes (for significant contributions)
- README (for major features)

Thank you for contributing to Echomind! 🎉
