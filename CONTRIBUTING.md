# Contributing to RustyAssets

Thank you for your interest in contributing to RustyAssets! This document provides guidelines and information for contributors.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Running Tests](#running-tests)
- [Code Standards](#code-standards)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (for PostgreSQL)
- [Git](https://git-scm.com/downloads)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/rusty-assets.git
   cd rusty-assets
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/nschoonbroodt/rusty-assets.git
   ```

## Development Environment

### Quick Setup

1. **Start the database:**
   ```bash
   docker-compose up -d
   ```

2. **Run database migrations:**
   ```bash
   sqlx migrate run --source crates/assets-core/migrations
   ```

3. **Build the project:**
   ```bash
   cargo build --workspace
   ```

4. **Create sample data (optional):**
   ```bash
   cargo run -- demo create-sample
   ```

### Environment Variables

Copy the example environment file:
```bash
cp .env.example .env
```

The default `DATABASE_URL` should work with the Docker setup:
```
DATABASE_URL=postgresql://rustyassets:rustyassets@127.0.0.1:5432/rustyassets
```

## Running Tests

### Basic Testing
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p assets-core
```

### Coverage Reports
```bash
# Generate HTML coverage report
cargo cov

# Open coverage report in browser
cargo cov-open

# Generate coverage for CI
cargo cov-ci
```

### Database Tests

Some tests use testcontainers and require Docker to be running:
```bash
# Make sure Docker is running first
docker --version

# Run database integration tests
cargo test database::tests::
```

## Code Standards

### Formatting and Linting

We use strict code quality standards:

```bash
# Format code (required before committing)
cargo fmt

# Check formatting
cargo fmt --check

# Run Clippy lints (all warnings are denied)
cargo clippy --all-targets --all-features -- -D warnings
```

### Code Style

- **Follow existing patterns** in the codebase
- **Use descriptive variable names**
- **Keep functions focused** and reasonably sized
- **Prefer PostgreSQL functions** over Rust computation where appropriate
- **Add comments** for complex algorithms or business logic where helpful

### Architecture Guidelines

- **Database-first**: Leverage PostgreSQL views and functions for computations
- **Double-entry bookkeeping**: Maintain accounting principles
- **Service layer**: Business logic goes in `assets-core/src/services/`
- **CLI layer**: User interface in `assets-cli/src/commands/`

## Commit Guidelines

### Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `ci`: CI/CD changes

### Examples
```
feat(accounts): Add hierarchical account creation
fix(import): Handle malformed CSV files gracefully
docs: Update README with Docker setup instructions
test(database): Add integration tests for migrations
```

## Pull Request Process

### Before Submitting

1. **Update your branch:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run quality checks:**
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```

3. **Test coverage:**
   ```bash
   cargo cov
   # Ensure new code has reasonable coverage
   ```

### PR Guidelines

- **Create focused PRs** that address a single issue or feature
- **Write clear descriptions** of what changes and why
- **Link to relevant issues** using `Fixes #123` or `Closes #123`
- **Include test coverage** for new functionality
- **Update documentation** if needed

### PR Template

The repository includes a PR template that will guide you through the submission process. Make sure to fill out all relevant sections.

## Issue Guidelines

### Before Creating an Issue

- **Search existing issues** to avoid duplicates
- **Check the README** and documentation first
- **Try the latest version** from the main branch

### Issue Types

Use the provided issue templates:

- **Bug Report**: For reporting bugs with reproduction steps
- **Feature Request**: For proposing new features
- **Question**: For general questions about usage

### Good Bug Reports Include

- Clear, descriptive title
- Steps to reproduce the problem
- Expected vs. actual behavior
- Environment details (OS, Rust version, etc.)
- Relevant logs or error messages

## Development Workflow

### Working on Issues

1. **Pick an issue** from the issue tracker
2. **Comment** that you're working on it
3. **Create a branch** with a descriptive name:
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes** following the guidelines above
5. **Submit a PR** when ready

### Testing Changes

Always test your changes thoroughly:

- Run the test suite
- Test with sample data: `cargo run -- demo create-sample`
- Try relevant CLI commands
- Check coverage reports

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and general discussion
- **README**: For basic setup and usage

## Code of Conduct

This project follows a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## License

By contributing to RustyAssets, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to RustyAssets! 🦀💰