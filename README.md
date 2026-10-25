# Stats Workbench

Reproducible statistical analysis for **Parquet, SQLite, and PostgreSQL** — from a CLI or a Rust SDK.

Stats Workbench is for developers and data workflows that need statistical computation to be **repeatable, scriptable, and machine-readable**, without tying the analysis to a notebook or spreadsheet.

## What it can do

- summary statistics
- Pearson / Spearman correlation
- distributions and histograms
- IQR-based outlier detection
- inferential statistics and confidence intervals
- linear regression and diagnostics
- time-series analysis
- TOML-based reproducible analysis recipes
- terminal, JSON, and Parquet output
- Parquet, read-only SQLite, and PostgreSQL inputs
- the same analysis engine through a Rust SDK
- optional [Insight Analytical Artifact v1 export](docs/analytical-export.md) with input/spec hashes and provenance
- optional Preact web UI

It produces statistical results and analytical artifacts. It does **not** turn those results into causal conclusions or business decisions.

## Install

### CLI

Requires Rust 1.75+.

```sh
git clone https://github.com/sibukixxx/stats-workbench.git
cd stats-workbench
cargo build -p sf-cli --release
./target/release/stats-workbench --help
```

### Rust SDK

After the crate is published:

```toml
[dependencies]
stats-workbench = "0.1"
```

The Rust import name is `stats_workbench`.

## Quick start

Generate the included local sample dataset:

```sh
cargo run -p stats-workbench --example generate_sample_data
```

Run summary statistics:

```sh
cargo run -p sf-cli -- eda summary \
  --source examples/data/sales.parquet \
  --cols price,quantity,revenue
```

Run a correlation:

```sh
cargo run -p sf-cli -- eda corr \
  --source examples/data/sales.parquet \
  --cols price,quantity,revenue \
  --method pearson
```

Run a reusable TOML recipe:

```sh
cargo run -p sf-cli -- run --recipe examples/05_full_pipeline.toml
```

## CLI

The installed binary is:

```text
stats-workbench
```

Main command groups:

```text
stats-workbench eda summary
stats-workbench eda corr
stats-workbench eda dist
stats-workbench stats outlier
stats-workbench infer ...
stats-workbench model ...
stats-workbench ts ...
stats-workbench run --recipe ...
```

The Cargo package that builds the binary remains `sf-cli`; `sf-*` crates are internal workspace components.

## Rust SDK

The public SDK crate is `stats-workbench`:

```rust
use stats_workbench::{output, source, step, StatsWorkbench};

#[tokio::main]
async fn main() -> stats_workbench::Result<()> {
    let source = source::parquet("data.parquet");
    let step = step::summary(["price", "quantity"]);

    let result = StatsWorkbench::run_step(&source, &step).await?;
    output::to_terminal(&result);
    Ok(())
}
```

Internal crates such as `sf-core`, `sf-data`, `sf-stats`, `sf-engine`, and `sf-report` implement the engine and are not the recommended integration point.

## When to use it

Use Stats Workbench when you need to:

- run the same statistical analysis repeatedly in CI or batch jobs;
- version the analysis definition alongside code;
- inspect structured data without setting up a notebook environment;
- generate machine-readable analytical results for another application;
- embed deterministic statistical analysis in a Rust application.

For exploratory research interpretation, hypothesis management, or evidence reasoning, use a downstream research system rather than adding those semantics here.

## Data sources

| Source | Support |
| --- | --- |
| Parquet | local files |
| SQLite | local, read-only |
| PostgreSQL | remote connections with TLS |

External sources and recipes should be treated as untrusted input. Paths and database access are bounded where applicable, and secrets should never be embedded in recipes committed to source control.

## Web UI

An optional Preact + Vite UI is available in `apps/web`.

```sh
cd apps/web
pnpm install
pnpm test
pnpm build
```

Cloudflare deployment notes are in [docs/deploy-cloudflare.md](docs/deploy-cloudflare.md).

## Development

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Web:

```sh
cd apps/web
pnpm test
pnpm build
```

## Repository structure

```text
crates/stats-workbench   public Rust SDK
crates/sf-cli            CLI package; builds the stats-workbench binary
crates/sf-engine         recipe and pipeline execution
crates/sf-stats          statistical computation
crates/sf-data           source loading
crates/sf-report         result formatting
crates/sf-core           shared types and contracts
apps/web                 optional browser UI
examples                 reproducible recipes and datasets
```

## Documentation

- [Examples](examples/README.md)
- [Cloudflare web deployment](docs/deploy-cloudflare.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)

## Scope

Stats Workbench owns deterministic statistical computation, recipe execution, bounded data loading, and machine-readable analytical results.

It does not own generic research reasoning, autonomous data acquisition, CRM/customer workflows, or business decisions.

## License

MIT. See [LICENSE](LICENSE).
