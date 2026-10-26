# Statistical portability — #6

StatForge owns computation; Insight owns research/evidence semantics. ClusterScope and EC Hub own their questions, tenant data and decisions. Export is optional and offline, without an Insight server, SDK, R or Quarto dependency.

## Audit of current main

| Existing owner | Implemented | Gap addressed here |
| --- | --- | --- |
| sf-stats | Summary/correlation/distribution, inference, regression, diagnostics, time series | No new formulas |
| sf-report JSON | Execution version/time and typed output values | No units, population/time/geography, input/spec hashes or research-ready semantics |
| Insight Analytical Artifact v1 | Dataset/spec identity, metrics/scalars, provenance, neutral Observation projection | Use unchanged contract; no parallel artifact system |
| Insight #117 | Design discussion of statistical semantics | Store this producer's explicit summary plan in existing parameters; do not declare full design implemented |

## First slice

`scripts/export_analytical.py` converts a selected **Summary** output from the existing StatForge JSON writer. It does not execute SQL, fetch URLs, invent statistics or choose methods. The caller supplies a reviewable versioned manifest, the exact input bytes and the exact recipe. Input/spec SHA-256 must match before export. Other result kinds fail explicitly until their semantics have their own reviewed adapter.

The manifest declares the question, variable dictionary (name/unit/definition), population, period, geography, missingness policy, method and assumptions. All exported columns must exist and have one declared comparable scope. Missing unit, variable, invalid counters, incompatible scopes and VIOLATED assumptions fail. UNKNOWN/NOT_CHECKED assumptions remain quality flags. A null numeric statistic becomes missing=true without a value; count and null_count are diagnostics. Descriptive summaries have no sampling interval: uncertainty is explicitly NOT_ESTIMATED, with a reason, never a zero-width fabricated interval. Quantiles remain StatForge's current rounded-order-statistic calculation.

Only selected summary values and validated plan metadata cross the boundary. Connection strings, SQL passwords, local file paths and unrelated artifacts are not copied from execution metadata. Each file has a byte limit. There is no outbound network in the exporter.

The result fingerprint hashes stable selected calculation values and tool/version, independently of execution time. ArtifactHash hashes the canonical exported JSON excluding artifactHash. Artifact ID derives from that hash: repeat export of the same run is byte-identical; a newly timed run has a new artifact ID while its result fingerprint should stay equal. Input/spec/plan fingerprints and source URLs permit a rerun. Hashes prove byte identity, not truth or successful execution; producer audit remains required.

## Public-data acceptance

`examples/insight-population` uses an existing, SHA-addressed e-Stat dashboard snapshot from Insight main: 2020 census population in three selected Ibaraki municipalities. It is a finite descriptive group, not a national sample or a causal estimate. Acquisition timestamp belongs to the saved source, not this rerun. Published date is unknown.

The runner normalizes the checked-in official snapshot to a local SQLite input, runs the existing sf recipe, exports, repeats computation, verifies stable result fingerprints, and optionally submits to a local Insight Public Engine v1. No external/customer data or credentials are needed. Tests cover golden values, bounded input, missing statistics, unit/scope/assumption rejection and privacy. Live source refresh is separate from deterministic acceptance.

Implementation and measured commands/results are recorded after execution; unsupported inference adapters remain follow-up scope.

## Historical validation before migration

The following measurements are from former StatForge PR #8. They are not validation of the current Stats Workbench branch. Current migration checks are recorded in migration-audit.md.

Base StatForge main: `64169a1219cca55c31c7a90514a50e50ff159ac1`. Engine code commit, tool version and no-random-seed declaration are recorded in parameters. Producer assumptions/units are caller-reviewed metadata, not verified statistical truth.

- Rust 1.99.0: `cargo build --workspace` and `cargo test --workspace` succeeded (207 tests). No Rust formulas or crate dependencies changed.
- Python 3.12: exporter regression tests succeeded (5 cases, including malformed/missing/nonfinite/scope/hash/assumption/privacy cases).
- Public-data runner: two real CLI computations succeeded; result fingerprint `a28cc1c5ccabe976e178b8cf1797e01ec585245e1b48b68745d217ae531b5af0` matched.
- Real local Insight delivery binary + real sf binary HTTP E2E succeeded (1 case): idempotent artifact receipt, deterministic completion, 7 evidence-linked observations, no manufactured Findings, report-ready bundle.

Run the self-contained real binary test in one process environment:

```sh
INSIGHT_BIN=/absolute/path/to/insight-lab STATFORGE_BIN=/absolute/path/to/sf python examples/insight-population/test_e2e.py
```

Without these explicit binaries the E2E is skipped, not passed. No live source refresh, managed remote authorization or browser UI was validated by this test. Summary is the only exported result kind; inferential, regression and forecasting outputs require reviewed semantics before supporting them.

Next: [StatForge #7](https://github.com/sibukixxx/StatForge/issues/7) adds an existing confidence-interval result after [Insight #117](https://github.com/sibukixxx/insight/issues/117) semantics review.

## Canonical producer

The CLI is `stats-workbench`. The exporter accepts both former StatForge run JSON and current `Stats Workbench` JSON, preserving the legacy producer/spec/ID for former inputs. Current inputs use `StatsWorkbench-summary-export`, `STATS_WORKBENCH_RECIPE` and `stats-workbench-` IDs. The existing Analytical Artifact v1 contract is unchanged.

The public population runner requires `--engine-commit` for the source SHA of the tested binary. The optional receiving-engine test requires INSIGHT_BIN, STATS_WORKBENCH_BIN and STATS_WORKBENCH_COMMIT. A skipped test is not a passing acceptance test.
