# Contributing to Stats Workbench

Thanks for your interest! We welcome contributions from the community.

## Development Setup

```bash
git clone https://github.com/sibukixxx/stats-workbench.git
cd Stats Workbench
cargo build
cargo test
```

## Code Style

- Run `cargo fmt` before committing
- Use `cargo clippy --all-targets` to catch common mistakes
- Keep changes focused; one feature or fix per PR

## Testing

- Add tests for new functionality
- Ensure all tests pass: `cargo test --all`
- Integration tests go in `crates/sf-cli/tests/`

## Commit Messages

- Use clear, descriptive commit messages
- Reference issues when relevant (e.g., "Fixes #123")
- Follow conventional commits if possible (`feat:`, `fix:`, `refactor:`, etc.)

## PR Guidelines

1. Fork the repository
2. Create a branch for your changes
3. Ensure tests pass locally
4. Open a PR with a clear description of changes
5. Respond to review feedback

## Reporting Issues

- Check existing issues before opening a new one
- Include reproducible steps and relevant context
- Mention your Rust version and platform

## License

By contributing, you agree that your code will be licensed under the MIT License.
