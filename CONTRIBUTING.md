# Contributing to Scedge Core

Thank you for your interest in contributing to Scedge Core! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Redis 7+
- Docker (optional, for testing)
- Git

### Setting Up Your Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/scedge.git
   cd scedge
   ```

3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/memophor/scedge.git
   ```

4. Install development dependencies:
   ```bash
   cargo build
   ```

5. Start Redis for local development:
   ```bash
   docker run -d --name redis -p 6379:6379 redis:7
   ```

## Development Workflow

### Running the Project

```bash
# Run with default settings
cargo run

# Run with custom configuration
SCEDGE_PORT=8080 SCEDGE_REDIS_URL=redis://localhost:6379 cargo run

# Run with hot-reload (install cargo-watch first)
cargo watch -x run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests only
cargo test --test '*'
```

### Code Quality

Before submitting a pull request, ensure your code meets our quality standards:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for security issues (install cargo-audit first)
cargo audit

# Build in release mode to check for optimization issues
cargo build --release
```

## Making Changes

### Branch Naming

Use descriptive branch names following these patterns:
- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test additions or modifications

### Commit Messages

Write clear, concise commit messages following this format:

```
<type>: <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test additions or modifications
- `chore`: Build process or auxiliary tool changes

Example:
```
feat: add support for Redis Cluster

Implement Redis Cluster support to enable horizontal scaling
of the cache backend. This allows Scedge to handle larger
workloads across multiple Redis nodes.

Closes #123
```

### Code Style

- Follow Rust standard style guidelines (enforced by `cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and concise
- Write documentation comments for public APIs

### Testing Requirements

- Add unit tests for new functions and modules
- Add integration tests for new API endpoints
- Ensure all tests pass before submitting PR
- Aim for >80% code coverage for new code

## Pull Request Process

1. **Update your fork** with the latest upstream changes:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Make your changes** in a new branch:
   ```bash
   git checkout -b feature/my-new-feature
   ```

3. **Test your changes** thoroughly:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit your changes** with clear commit messages:
   ```bash
   git add .
   git commit -S -m "feat: add my new feature"
   ```

5. **Push to your fork**:
   ```bash
   git push origin feature/my-new-feature
   ```

6. **Create a Pull Request** on GitHub with:
   - Clear title describing the change
   - Detailed description of what changed and why
   - Reference to any related issues
   - Screenshots/examples if applicable

7. **Code Review Process**:
   - Maintainers will review your PR
   - Address any requested changes
   - Once approved, your PR will be merged

## Signing Your Commits

We require all commits to be signed with GPG:

```bash
# Configure Git to sign commits
git config --global user.signingkey YOUR_GPG_KEY_ID
git config --global commit.gpgsign true

# Commit with signature
git commit -S -m "your message"
```

## Areas for Contribution

We welcome contributions in these areas:

### High Priority
- [ ] Additional cache backends (RocksDB, DynamoDB, etc.)
- [ ] ANN-based semantic similarity search
- [ ] Performance optimizations
- [ ] Enhanced metrics and observability

### Medium Priority
- [ ] Additional authentication methods
- [ ] Rate limiting and throttling
- [ ] Cache warming strategies
- [ ] Documentation improvements

### Good First Issues
Look for issues labeled `good-first-issue` on GitHub. These are typically:
- Documentation improvements
- Small bug fixes
- Test additions
- Example code

## Documentation

- Update README.md if you change functionality
- Add inline documentation for public APIs
- Update examples if you add new features
- Add migration guides for breaking changes

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Check existing issues and discussions first

## License

By contributing to Scedge Core, you agree that your contributions will be licensed under the Apache 2.0 License.

---

Thank you for contributing to Scedge Core! Your efforts help make edge caching better for everyone.
