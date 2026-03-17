# Contributing to Transf-ORM-CLI

Thank you for your interest in contributing to Transf-ORM-CLI! We welcome contributions from the community. This document provides guidelines and instructions for contributing.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Code Guidelines](#code-guidelines)
- [Reporting Issues](#reporting-issues)

## Getting Started

1. **Fork the repository** - Click the "Fork" button on GitHub
2. **Clone your fork** - `git clone https://github.com/YOUR-USERNAME/Transf-ORM-CLI.git`
3. **Add upstream remote** - `git remote add upstream https://github.com/Transf-ORM/Transf-ORM-CLI.git`

## Development Setup

### Prerequisites

- Rust 1.70.0 or later
- Cargo
- Git

### Setting up your environment

```bash
# Clone and navigate to the project
git clone https://github.com/YOUR-USERNAME/Transf-ORM-CLI.git
cd Transf-ORM-CLI

# Build the project
cargo build

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run clippy (linter)
cargo clippy
```

## Making Changes

1. **Create a feature branch** - `git checkout -b feature/your-feature-name`
2. **Make your changes** - Follow the code guidelines below
3. **Keep commits clean** - Use clear, descriptive commit messages
4. **Keep your branch updated** - `git rebase upstream/main`

## Testing

All contributions must include tests. Before submitting:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Test a specific module
cargo test module_name
```

## Submitting Changes

1. **Push to your fork** - `git push origin feature/your-feature-name`
2. **Create a Pull Request** - Go to GitHub and click "New Pull Request"
3. **Fill in the PR template** - Provide a clear description of your changes
4. **Wait for review** - Maintainers will review your contribution

### PR Guidelines

- Link related issues with `Closes #issue-number`
- Provide a clear description of what and why you changed
- Define **Acceptance Criteria** with checkboxes:

  ```markdown
  ## Acceptance Criteria
  
  - [ ] Criterion 1
  - [ ] Criterion 2
  - [ ] Criterion 3
  ```

- Keep PRs focused on a single feature or fix
- Ensure all CI checks pass

### What are Acceptance Criteria?

Acceptance Criteria are clear, testable conditions that must be met for the PR to be considered complete. They help reviewers understand exactly what the PR accomplishes.

## Code Guidelines

### Rust Style

- Follow the Rust naming conventions
- Use `cargo fmt` to format code automatically
- Run `cargo clippy` and fix all warnings
- Add documentation comments for public items

### Documentation

We use **Doxygen** for function documentation. Please follow this format for all public items:

```rust
/// @brief Brief description of what this function does
///
/// More detailed explanation of the function's purpose and behavior.
///
/// @param param1 Description of the first parameter
/// @param param2 Description of the second parameter
///
/// @return Description of the return value
///
/// @note Any important notes or warnings
///
/// @example
/// ```
/// let result = my_function(value1, value2);
/// ```
pub fn my_function(param1: Type, param2: Type) -> ReturnType {
    // implementation
}
```

**Doxygen tags**:

- `@brief` - One-line description
- `@param` - Document parameters
- `@return` - Document return value
- `@note` - Add notes or warnings
- `@example` - Provide usage examples
- `@deprecated` - Mark as deprecated
- `@todo` - Mark incomplete implementations

### Commit Messages

We follow the **Karma convention** for commit messages. This format ensures clear, semantic commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type** (required):

- `feat` - A new feature
- `fix` - A bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, semicolons, etc.)
- `refactor` - Code refactoring without changing functionality
- `perf` - Performance improvements
- `test` - Adding or updating tests
- `chore` - Build process, dependencies, or tooling changes
- `ci` - CI/CD configuration changes

**Scope** (optional):

- Component or module name (e.g., `cli`, `parser`, `schema`, `transform`)

**Subject** (required):

- Use the imperative mood ("add" not "added")
- Use lowercase
- No period at the end
- Max 50 characters

**Body** (optional):

- Explain what and why, not how
- Wrap at 72 characters
- Separate from subject with a blank line

**Footer** (optional):

- Reference issues: `Closes #123`
- Breaking changes: `BREAKING CHANGE: description`

**Examples**:

```
feat(cli): add support for PostgreSQL schemas

Add PostgreSQL schema transformation support to the CLI.
This allows users to directly work with PostgreSQL-specific
schema features.

Closes #42
```

```
fix(parser): resolve recursive schema parsing error

The parser was not correctly handling deeply nested schemas.
This fix implements proper depth tracking.

Closes #156
BREAKING CHANGE: Schema.parse() now requires explicit depth limit
```

```
docs(readme): update installation instructions
```

## Reporting Issues

Found a bug? Here's how to report it:

1. **Check existing issues** - Avoid creating duplicates
2. **Create a new issue** - Click "New Issue" on GitHub
3. **Provide details**:
   - Clear title
   - Detailed description
   - Steps to reproduce
   - Expected vs actual behavior
   - Your environment (OS, Rust version, etc.)

## Security Issues

**Do not** open a public issue for security vulnerabilities. Please follow our [Security Policy](SECURITY.md) instead.

## License

By contributing to Transf-ORM-CLI, you agree that your contributions will be licensed under its license.

## Questions?

Feel free to:

- Open a discussion on GitHub
- Create an issue for clarification
- Check our wiki for documentation
