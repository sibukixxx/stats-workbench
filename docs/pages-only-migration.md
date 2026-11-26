# Pages-only migration — 2026-11-26

## Decision
Retire ChatGPT Sites immediately, preserve GitHub source/history, and use Cloudflare Pages static hosting only. No Workers (including static-assets Workers), Pages Functions, Containers, D1, R2, paid external APIs, or new billing commitments. No new scheduled jobs or deployment workflow. Keep existing Cloudflare services and Access policies unchanged until inventory and validation are complete.

## Retirement gate (urgent, independent of migration)
- Owner opens Sites in ChatGPT web/desktop and deletes only the named site, after checking that no necessary data exists; export any needed data locally first. Sharing is already owner-only and is not retirement. Deletion is irreversible. Verify the old URL is no longer served and check any associated schedules, resources and billing status. Do not mark complete until verified. GitHub changes do not stop a Site. Do not deploy it again.
- Official instructions: https://help.openai.com/ja-jp/articles/20001339-creating-and-using-chatgpt-sites

## Cost and deployment guard
- Pages Git integration or static upload only; free static requests under current published pricing: https://developers.cloudflare.com/pages/functions/pricing/ . Free-plan build/file limits still apply; no promise of future pricing.
- Build only when needed; no extra scheduled CI. Fail build if output contains _worker.js, server entrypoint or Functions; verify no Functions or resource bindings in Pages settings. Never put tokens or credentials in browser assets.
- Preserve existing private access (including preview and pages.dev endpoints); verify owner access and anonymous/non-owner denial before switching URLs. If cannot guarantee this, do not publish.
- Acceptance: frozen install, existing unit/type/build checks, mobile/desktop smoke, no network requests to API/Sites, no Worker/Functions invocations, verify Cloudflare project configuration and old Site retirement. No test or deployment claimed by this design document.

## Stats Workbench
- Retire https://techvit-stats-workbench.steadymyna13.chatgpt.site (v4). No server data/bindings recorded.
- Existing Pages project techvit-stats-private at stats.techvit.me is documented in docs/deploy-sites.md. Verify live account and project, reuse it, do not create duplicates or change Access. Root apps/web, build corepack enable && pnpm install --frozen-lockfile && pnpm build, output dist, main branch. Preserve pnpm 10.28.0 and Vite 6.
- Existing Preact demo in apps/web/src/state.ts uses sample builders. Keep UI and explicit demo disclosure, all five mock analysis operations and direct route reloads (Pages SPA fallback). Keep Rust CLI/SDK unchanged. For genuine local analysis, a separate opt-in future milestone can add browser File API + WASM or pure JS, with parity tests; no fake results, remote DB or secrets. Do not imply real analysis is implemented.
- Remove active Sites deployment manifest and update README/deployment docs, preserving historical evidence; retire alternate wrangler static-assets Worker config and deploy:cf script after checking consumers; do not delete live Cloudflare resources without inventory. Use existing Pages Git build only. Do not trigger unnecessary production builds for design-only changes.
- Tests: pnpm test, pnpm typecheck, pnpm build, direct /run and all demo routes, phone widths, owner-only Access and no Functions bindings. Confirm current Pages project and actual deployment separately.
