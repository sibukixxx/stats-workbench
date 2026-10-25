# Migration audit — 2026-10-25

Canonical repository: stats-workbench. Source snapshots: StatForge `5c3078af1ec74ee0cbbb4f3613fef6d901e0a5b9` (2026-10-11), stats-workbench `f38f676b36db15cac333f488399461fc4c668c2b` (2026-10-07).

The earlier migration renamed the SDK directory, Rust import and CLI. Engine code is retained; old names are not interchangeable with missing functionality. The former repository advanced afterward in PR #8: its offline Summary-to-Analytical-Artifact exporter, reviewed statistical plan, exact e-Stat snapshot and producer/receiver harness were missing from the canonical tree. These assets are restored here without duplicating formulas.

## Implemented

- Existing StatForge Summary JSON and canonical Stats Workbench JSON are accepted.
- Former inputs retain legacy producer/spec/ID naming. Current inputs use canonical naming; Analytical Artifact schemaVersion remains 1.
- Input/spec hashes, stable numerical fingerprint, explicit missingness/uncertainty/assumptions and source provenance are retained.
- Population acceptance requires the tested binary source SHA, replacing the former hard-coded engine commit.
- Export tests are included in `make release-check`.

## Current verification

- 2026-10-25: `make test-analytical` PASS: 6 exporter tests and 1 official-snapshot/SQLite reproduction test.
- Rust fmt/clippy/workspace tests/build, web install/test/build, current dependency audits and real Insight HTTP acceptance require a fresh run against this branch before release.
- Prior session results and former StatForge PR #8 are historical evidence, not a release pass for this commit. The temporary environment was reset before those changes/logs were persisted.

## Release constraints

The repository remains Private until the full release gate passes. Anonymous clone, public package resolution, initial tag/release and official-site Available status are not verified. No shared history was rewritten. Existing release issue #3 includes an author-display-name decision that remains open; commit email addresses and all public refs must be reviewed before visibility changes.

Existing internal sf-* crate layering is retained. Insight owns receiving/research semantics; historical consumer fixtures keep their original producer identity. Confidence-interval export remains separate follow-up work, rather than an implemented feature of this migration.
