# TechVit owner-only Cloudflare Pages release (2026-10-26)

This note describes **TechVit internal dogfooding**, not the public Rust SDK / CLI distribution.

- Cloudflare Pages project: `techvit-stats-private`, GitHub repository `sibukixxx/stats-workbench`, production branch `main`.
- Root: `apps/web`, command: `corepack enable && pnpm install --frozen-lockfile && pnpm build`, output: `dist`.
- Reserved custom hostname: `stats.techvit.me`. Protected through Cloudflare Access exact owner email. Also protect `techvit-stats-private.pages.dev` and `*.techvit-stats-private.pages.dev` to prevent Pages origin/deployment URL bypass.
- Preview branch builds disabled. Do not treat `workers_dev:false` (the alternative Workers deployment config) as protection of Pages deployments.
- The current dashboard is a **static mock-data visualization**. This deploy does not run the Rust CLI / SQLite / PostgreSQL connectors in Cloudflare.

## Verification gates

1. Confirm Cloudflare Pages production deployment succeeded with the expected `main` commit SHA and green build logs.
2. Independently check unauthenticated access to **all three host surfaces** fails, and the owner can log in to `stats.techvit.me` using Cloudflare Access.
3. Test dashboard rendering and basic interactive charts in a browser. Do not record PASS before real execution.
4. Pin the last known-good Pages deployment and document rollback via Pages rollback.

Secrets must never be embedded in frontend static assets.
