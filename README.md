# StatForge

<!-- oss-readme-intro:v1 -->
A reproducible **statistical analysis workbench** for Parquet, SQLite and PostgreSQL. Run common statistics from the CLI, define repeatable analyses as TOML recipes, or embed the same engine as a Rust SDK.

### Who is it for?

- developers who need deterministic statistics without building a full notebook stack
- data pipelines that need versionable analysis recipes and machine-readable output
- Rust applications that need reusable statistical primitives

### Quick start

```sh
git clone https://github.com/sibukixxx/StatForge.git
cd StatForge

cargo build -p sf-cli --release

cargo run -p sf-cli -- eda summary \
  --source examples/data/sales.parquet \
  --cols price,quantity,revenue
```

### What can it do?

Summary statistics, Pearson/Spearman correlation, distributions, histograms and IQR outlier detection; load from Parquet/read-only SQLite/PostgreSQL; run TOML recipes; export terminal, JSON or Parquet results; and expose the same functionality through a Rust SDK.

### Use it when

Use it when you want an analysis to be **repeatable, scriptable and embeddable**, rather than tied to a one-off spreadsheet or notebook session.

See [Examples & Recipes](#examples--recipes), [Web Dashboard](#web-dashboard), and [SDK packaging](#sdk-packaging) for deeper usage.
<!-- /oss-readme-intro -->


> [!NOTE]
> **Repository transition:** this repository remains the canonical source until the planned migration to `stats-workbench`. The replacement repository will be created from cleaned history after verification. Until then, commands, binary names and links intentionally use the current repository/product names.


<!-- ecosystem-expansion:v1 -->
## Expansion contract

This repository is one capability owner, vertical, experiment, or reference asset in the TechVit ecosystem. New ideas should extend the existing graph before creating another silo.

**Canonical responsibility:** deterministic statistical computation / analytical artifact producer

When new work appears:

1. **Extend here** only when the behavior belongs to this repository's stated responsibility.
2. **Integrate, don't copy** when another repository owns the concept. Exchange versioned API/artifact/event references; do not share databases or import another product's internals.
3. **Promote reusable capability only after real reuse appears.** Keep the first vertical slice close to its product; extract a common core when there are at least two real consumers or a deliberate public/private boundary.
4. **Create a new repository only** for a distinct product/vertical with its own lifecycle, deployment/security boundary, or OSS/commercial boundary.
5. **Old experiments remain assets.** Move validated rules, contracts, fixtures, datasets, and learnings into the canonical owner before freezing or archiving duplicate implementations.

**Route new work:** 統計計算・recipe・machine-readable resultはここへ、Research interpretationはInsight、business decisionはconsumerへ渡す。二つ以上のconsumerで使われる分析primitiveだけをSDKとして安定化する。

Cross-cutting owners: TechVit Admin = company operations; TechVit Platform = shared cloud/bootstrap/identity; Insight = research semantics; techvit-insight = managed research; Relayboard = work execution; AI Allowance Guard = privileged-action authority; EC Hub = commerce; AffiliateMediaOS/VideoForge = content production; ja-company-* = company-data/outreach producers; public/product sites = acquisition surfaces.

If ownership is unclear, implement the smallest vertical slice in the nearest existing owner first. Extract only when actual reuse proves the boundary.
<!-- /ecosystem-expansion:v1 -->

**Reproducible statistical analysis on Parquet, SQLite, and PostgreSQL — as CLI or Rust SDK.**

Run exploratory analysis interactively or define reusable pipelines as TOML recipes. StatForge handles data loading, transformation, statistical computation, and output formatting — with security hardening for untrusted data sources.


<!-- current-implementation:2026-10-06 -->
## Current implementation snapshot — 2026-10-06

- The implemented product is a reproducible statistical-computation workbench for Parquet, read-only SQLite and TLS PostgreSQL sources, available as a Rust CLI, public Rust SDK facade and optional Preact web dashboard.
- Implemented analysis primitives include summaries, Pearson/Spearman correlation, distributions/histograms and IQR outlier detection; TOML recipes make multi-step analyses versionable and repeatable.
- Results can be emitted to terminal, JSON or Parquet for downstream evidence/research consumers; this repository deliberately does not turn statistical results into causal claims or business decisions.
- Security hardening treats recipes and external data sources as untrusted: database access is bounded/read-only where applicable, paths are validated and secrets are redacted.
- The cleaned successor is planned as `stats-workbench`; current crate names (`statforge`, `sf-*`) remain unchanged until migration and package-release planning.
<!-- /current-implementation -->

<!-- role-boundary:v1 -->
## Role and boundaries

**Role:** structured dataに対する再現可能なdeterministic statistical computationを提供する **analytical producer / Rust SDK + CLI**。

### Owns

- data-source loading and bounded transformations
- statistical computation
- versionable recipe execution
- machine-readable deterministic results

### Does not own

- causal/business interpretation
- generic research/hypothesis semantics
- customer workflow or CRM
- autonomous source acquisition
- external privileged actions

### Integration

```text
Parquet / SQLite / PostgreSQL
        ↓
      StatForge
        ↓ deterministic analytical result
research engine / domain application / human
```

Statistical resultを原因やbusiness actionへ自動変換しない。必要なconsumerはprovenance付きresultをEvidence/Analytical Artifactとして利用する。
## Installation

### Prerequisites
- Rust 1.75+ ([install](https://rustup.rs/))
- For database sources: SQLite or PostgreSQL client libraries

### Build from Source

```bash
git clone https://github.com/sibukixxx/StatForge.git
cd StatForge
cargo build -p sf-cli --release
```

The CLI binary is at `target/release/sf-cli`.

## Quick Start

### 1. Generate Sample Data

```bash
cargo run -p statforge --example generate_sample_data
```

This creates `examples/data/sales.parquet` (no external downloads needed).

### 2. Run EDA Commands

```bash
# Summary statistics
cargo run -p sf-cli -- eda summary \
  --source examples/data/sales.parquet \
  --cols price,quantity,revenue

# Correlation matrix
cargo run -p sf-cli -- eda corr \
  --source examples/data/sales.parquet \
  --cols price,quantity,revenue \
  --method pearson

# Outlier detection (IQR-based)
cargo run -p sf-cli -- stats outlier \
  --source examples/data/sales.parquet \
  --cols revenue
```

### 3. Use the Rust SDK

Add to your `Cargo.toml`:

```toml
[dependencies]
statforge = "0.1"
```

Then in your code:

```rust
use statforge::{output, source, step, StatForge};

#[tokio::main]
async fn main() -> statforge::Result<()> {
    let data = source::parquet("data.parquet");
    let summary = step::summary(["price", "quantity"]);
    
    let result = StatForge::run_step(&data, &summary).await?;
    output::to_terminal(&result);
    Ok(())
}
```

### 4. Run Recipes

Define a TOML pipeline:

```toml
[source]
type = "parquet"
path = "data/sales.parquet"

[[steps]]
id = "summary"
type = "eda.summary"
cols = ["price", "quantity", "revenue"]

[output]
terminal = true
format = "json"
path = "output.json"
```

Execute it:

```bash
cargo run -p sf-cli -- run --recipe recipe.toml
```

## Use Cases

- **Ad-hoc analysis** — Quickly explore data without writing code
- **Pipeline automation** — Define repeatable analyses as code (TOML recipes)
- **Data quality checks** — Outlier detection, correlation analysis
- **Integration** — Use as a library in Rust applications
- **Batch processing** — Generate JSON/Parquet outputs for downstream systems

## Features

- 📊 **Summary statistics** — count, mean, std dev, quantiles, min/max
- 📈 **Correlations** — Pearson, Spearman methods
- 📉 **Distributions** — Histograms and density plots
- 🎯 **Outlier detection** — IQR-based method
- 🔐 **Security hardening** — Read-only DB access, path validation, secret redaction
- 🚀 **Operational hardening** — TLS for remote databases and static-binary support

## Supported Data Sources

- **Parquet** — Local files
- **SQLite** — Local files (read-only)
- **PostgreSQL** — Remote connections with TLS

## Output Formats

- Terminal (pretty-printed tables)
- JSON (machine-readable)
- Parquet (for chaining analyses)

## Examples & Recipes

See [examples/README.md](examples/README.md) for comprehensive examples and real-world use cases:

- [01_basic_summary.toml](examples/01_basic_summary.toml) — Basic summary statistics
- [02_correlation.toml](examples/02_correlation.toml) — Pearson / Spearman correlation
- [03_distribution.toml](examples/03_distribution.toml) — Histograms
- [04_outlier_detection.toml](examples/04_outlier_detection.toml) — IQR-based outlier detection
- [05_full_pipeline.toml](examples/05_full_pipeline.toml) — Complete pipeline with JSON output
- [bike_sharing/](examples/bike_sharing/) — Real dataset example (weather vs. bike rentals)
- [nyc_taxi/](examples/nyc_taxi/) — Taxi fare vs. distance analysis

## Web Dashboard

A Preact + Vite dashboard is included in `apps/web` for interactive exploration. Deploy to Cloudflare Workers Static Assets (free tier) with:

```bash
cd apps/web
pnpm install
pnpm build
pnpm deploy:cf
```

See [docs/deploy-cloudflare.md](docs/deploy-cloudflare.md) for detailed deployment instructions.

## Development

### Testing

```bash
cargo test --all
```

### Code Quality

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
```

### Architecture

StatForge is organized as a layered Rust workspace:

- **sf-cli** — CLI entry point
- **sf-engine** — Pipeline orchestration
- **sf-stats** — Statistical computations
- **sf-data** — Data loading (Parquet, SQLite, PostgreSQL)
- **sf-report** — Output formatting
- **sf-core** — Shared types and errors
- **statforge** — Public Rust SDK facade

See [AGENTS.md](AGENTS.md) for detailed architecture and conventions.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

For security issues, see [SECURITY.md](SECURITY.md).


## SDK packaging

- Public SDK crate: `statforge`
- Internal implementation crates: `sf-core`, `sf-data`, `sf-stats`, `sf-engine`, `sf-report`
- CLI crate: `sf-cli`
- Recommended publish order: `sf-core` -> `sf-data` / `sf-stats` / `sf-report` -> `sf-engine` -> `statforge` -> `sf-cli`

The recommended Rust integration point is the `statforge` crate. The `sf-*` crates remain available for workspace internals and lower-level integrations.
