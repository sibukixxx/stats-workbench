# Public census → Stats Workbench → Insight

This fixture reuses an existing official e-Stat snapshot, not synthetic population numbers. Upstream acquisition is pinned at Insight main commit `2b4b0a8088fbfe7f837e8dbb6dcf89d7ff4a4ed6`, `examples/demo-ja/official/raw/population_census_2015_2020.json` (SHA-256 `163f1816517f73f6d1641b43e79ac1663371ef1ab6c0227d4b5bcff1ff7e55a5`). Retrieved 2026-09-27T16:50:36.651Z; published date unknown.

Source: [Statistics Dashboard](https://dashboard.e-stat.go.jp/), Ministry of Internal Affairs and Communications, Population Census, indicator `0201010000000010000`. The original API request is:

```
https://dashboard.e-stat.go.jp/api/1.0/Json/getData?Lang=JP&IndicatorCode=0201010000000010000&RegionCode=08201,08212,08220&Cycle=3&IsSeasonalAdjustment=1&TimeFrom=2015CY00&TimeTo=2020CY00&MetaGetFlg=Y
```

License/attribution: [Public Data License 1.0](https://dashboard.e-stat.go.jp/static/terms). Source: Statistics Dashboard (https://dashboard.e-stat.go.jp/). Data processed from the Statistics Dashboard. This service uses its API; the Government does not guarantee this service's contents.

## Reviewed question and plan

Describe the distribution of 2020 census resident population across **Mito, Hitachiota and Tsukuba**, as of 2020-10-01. These three municipalities are a selected finite group. The mean is the mean of three municipal population counts, not a population-weighted mean, national estimate or average individual characteristic. The input includes 2015/2020; the recipe explicitly filters 2020 and orders region codes. No cross-year comparison is performed.

Known numerical checks: count=3, missing=0, mean=186981 persons, minimum=48602, maximum=270685, median=241656. The source figures are 270685, 48602 and 241656. The existing Rust Summary implementation owns all calculations, including sample standard deviation (ddof=1) and rounded order-statistic quantiles. This descriptive dispersion is **not** a sampling standard error or confidence interval.

The source includes unspecified nationality and age. Geography/code continuity does not establish unchanged boundaries; this assumption is UNKNOWN. No cause, policy effect, forecast or business recommendation follows from these figures.

## Run

Python 3.10+ (standard library) and the existing Rust `stats-workbench` binary are needed. No R, Quarto, model API key or Insight service is required for export.

```sh
cargo build --workspace
python scripts/test_export_analytical.py
python examples/insight-population/run.py --sf target/debug/stats-workbench --engine-commit <tested-source-SHA> --out .local/population-acceptance
```

Use a fresh output directory for every acceptance run. The runner refuses an existing input database rather than deleting it. Outputs include manifest, exact normalized SQLite input, both raw Stats Workbench results, sealed Analytical Artifact and report-ready JSON with fingerprints and limitations.

To test the actual receiving engine, build/start Insight separately against a new test database:

```sh
# in Insight
make build
./bin/insight-lab serve -host 127.0.0.1 -port 18787 -no-browser -no-web -db /path/to/new-test.db
# in Stats Workbench
python examples/insight-population/run.py --sf target/debug/stats-workbench --engine-commit <tested-source-SHA> --out .local/population-with-insight --insight http://127.0.0.1:18787
```

A self-contained E2E is also available: `INSIGHT_BIN=/absolute/path/to/insight-lab STATS_WORKBENCH_BIN=/absolute/path/to/stats-workbench STATS_WORKBENCH_COMMIT=<tested-source-SHA> python examples/insight-population/test_e2e.py`. It starts and terminates its own local engine with a fresh temporary database. Missing binary variables produce a skip, not a success.

The optional HTTP run creates a new opaque acceptance subject, adds the exported artifact twice with the same idempotency key, starts deterministic DATASET_ANALYSIS, polls completion and verifies observation/document linkage. It uses Public Engine v1 only. The exporter never connects to the server. Local acceptance is not SaaS/staging authorization testing.

## Measured validation

Results and limitations are recorded in `../../docs/analytical-export.md`. For another producer result, use `scripts/export_analytical.py --result ... --manifest ... --input ... --recipe ... --out ...`; the manifest requires the exact input/spec SHA-256, source attribution, caller-reviewed dictionary/scope and engine code commit.
