# Contributing to Stats Workbench

Thanks for considering a contribution.

## Development setup

```sh
git clone https://github.com/sibukixxx/stats-workbench.git
cd stats-workbench
cargo build --workspace
cargo test --workspace
```

For the web UI:

```sh
cd apps/web
pnpm install
pnpm test
pnpm build
```

## Before opening a pull request

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Keep changes focused. Statistical behavior changes should include deterministic regression coverage or fixtures with independently checkable expected values.

Do not commit credentials, private datasets, database connection strings, or AI session/attribution metadata.

## Scope

Good contributions include statistical primitives, data-source compatibility, deterministic recipe execution, output formats, CLI/SDK usability, tests, examples, and security hardening.

Business recommendations, generic research reasoning, CRM/customer workflows, and autonomous source acquisition are outside this repository.

## License

By contributing, you agree that your contribution is licensed under the MIT License.
