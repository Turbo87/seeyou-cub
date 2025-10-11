# Contributing Guide

Thank you for your interest in contributing to this project!

## Welcome

seeyou-cub is a Rust library for reading and writing the SeeYou CUB binary file format, which stores airspace data for flight navigation software. We welcome contributions of all kinds: bug reports, feature proposals, documentation improvements, and code contributions.

Please be respectful and constructive in all interactions with the project and its community.

### Key Resources

- [README](README.md) - Project overview and usage examples
- [CUB File Format Documentation](docs/CUB_file_format.md) - Detailed format specification
- [GitHub Issues](https://github.com/Turbo87/seeyou-cub/issues) - Bug reports and approved features
- [GitHub Discussions](https://github.com/Turbo87/seeyou-cub/discussions) - Feature proposals and questions

## Reporting Issues and Proposing Features

We use a tiered approach for different types of contributions:

### Bug Reports

If you've found a bug, please report it directly as a [GitHub Issue](https://github.com/Turbo87/seeyou-cub/issues/new). Include:

- Clear description of the problem
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Environment details (Rust version, OS)
- Minimal code example demonstrating the issue
- Reference to a specific CUB file that triggers the bug (if applicable)

### Feature Requests

Feature proposals should start as a [GitHub Discussion](https://github.com/Turbo87/seeyou-cub/discussions/new). Please include:

- Description of the use case
- Explanation of why the feature would be valuable
- Optional: suggested implementation approach

Once a maintainer approves the proposal and agrees it fits the project's scope, it will be converted to a GitHub Issue for tracking implementation.

### Questions

Questions about usage, clarification, or general discussion should go to [GitHub Discussions](https://github.com/Turbo87/seeyou-cub/discussions/new) rather than Issues.

## Development Setup and Workflow

### Prerequisites

- Latest stable Rust toolchain (`rustup update stable`)
- Git

### Fork-Based Workflow

1. Fork the repository on GitHub
2. Clone your fork locally: `git clone https://github.com/YOUR_USERNAME/seeyou-cub.git`
3. Create a feature branch with a descriptive name:
   - `git checkout -b fix-coordinate-conversion`
   - `git checkout -b add-timezone-support`
4. Make your changes on the feature branch
5. Push to your fork and submit a pull request

### Understanding the Project

Before contributing, familiarize yourself with:

- The two-tier API design (high-level and low-level APIs) described in the README
- The project structure by examining the codebase
- Existing code patterns by looking at similar files before creating new ones

The project follows standard Rust conventions without additional custom style requirements.

## Testing Requirements

All changes must include appropriate tests:

- **New features**: Include tests demonstrating the functionality works
- **Bug fixes**: Include tests that would have caught the bug

### Running Tests Locally

Before submitting a PR, run the full test suite:

```bash
# Run all tests
cargo test

# Run clippy linter
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Accept snapshot test changes (requires cargo-insta)
cargo insta accept
```

**Note**: Snapshot testing requires installing `cargo-insta` (see <https://insta.rs/docs/quickstart/#installation>).

The project uses the `france_2024.07.02.cub` fixture for integration tests. CI will run these same checks on all PRs, so ensure all checks pass locally first.

## Pull Request Guidelines

### What Makes a Good PR

- **Single concern**: Focus on one bug fix or one feature per PR
- **Clear title and description**: Explain what changed and why
- **Reference related items**: Link to related issues or discussions
- **Pass all CI checks**: Tests, clippy, and formatting must pass
- **Include tests**: All new functionality and bug fixes need tests
- **Follow existing patterns**: Match the style and structure of similar code

### PR Description

Your PR description should:

- Explain the approach taken, especially for non-obvious changes
- Reference the issue or discussion that motivated the change
- Note any areas where you'd like specific feedback

### Commit Messages

The project uses present tense imperative mood for commit messages:

- ✅ "Add support for timezone handling"
- ✅ "Fix coordinate conversion for anti-meridian"
- ✅ "Extract `validate_bounds()` function"
- ❌ "Added timezone support"
- ❌ "Fixed bug"

Keep messages technical and descriptive. See the git history for examples.

### Review and Merge Process

- Maintainers will review all pull requests
- Be responsive to feedback and willing to iterate on changes
- Maintainers may request changes or clarifications
- Once approved, a maintainer will merge the PR

## Questions and Getting Help

If you have questions about contributing, need help with your development environment, or want to discuss an approach before starting work, please use [GitHub Discussions](https://github.com/Turbo87/seeyou-cub/discussions/new).

We encourage you to ask questions early rather than spending time going in the wrong direction. Maintainers are happy to provide guidance on:

- Implementation approaches
- Testing strategies
- Project architecture
- Development environment setup

## Thank You

Thank you for taking the time to contribute to seeyou-cub. Your efforts help make this project better for everyone!
