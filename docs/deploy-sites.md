# ChatGPT Sites: TechVit Stats Workbench

## Scope and architecture

Owner-only static deployment of `apps/web`. Preact, Vite 6 and the existing design are retained. Node.js 22+ and the pinned pnpm are supported; no Vinext/server migration is needed for static assets. Build command remains `pnpm build` (TypeScript + Vite), output `dist`. The Site checkout uses `.openai/hosting.json` with `static.directory: "dist"`. No D1, R2, new authentication, secrets, connectors, or runtime backend are needed. Sites enforces access outside the application.

All screens use deterministic sample builders in `src/data.ts`, called by `src/state.ts`. Source settings and recipe text are demonstrations; files, database connections and TOML are not executed. Do not enter customer data, credentials, or connection secrets. State is in browser memory and resets on reload. Future real-engine integration belongs behind the execution functions in `state.ts`, using the existing result types; do not duplicate Rust statistical algorithms in the UI.

## Existing Cloudflare environment (read-only verification, 2026-11-01)

The active service is Pages `techvit-stats-private`, with `stats.techvit.me` and `techvit-stats-private.pages.dev`. GitHub `sibukixxx/stats-workbench`, production branch `main`, root `apps/web`, command `corepack enable && pnpm install --frozen-lockfile && pnpm build`, output `dist`. Preview deployments are disabled. The current production source is `a17b67c7d7b50f02aa93044d851afb34e46f561c`.

`deploy-cloudflare.md` describes an alternative Workers Static Assets deployment, not this actual Pages project. Its example `workers.dev` URL also requires enabling `workers_dev`, currently false in Wrangler. Neither the Pages settings nor Wrangler is changed for Sites.

To preserve the current Cloudflare deployment, prefix the GitHub squash merge commit with `[CF-Pages-Skip]`. Cloudflare documents this prefix as skipping a Pages build. Do not disable Git integration or change Access/secrets to achieve this.

## Update procedure

1. Check latest GitHub main, AGENTS.md and the existing Site identity/access. Work on a branch. Use Node.js 22+ and pnpm from `packageManager`.
2. In `apps/web`: `pnpm install --frozen-lockfile`, `pnpm test`, `pnpm exec tsc --noEmit`, `pnpm build`. Verify the existing Wrangler build with `pnpm exec wrangler deploy --dry-run` only.
3. Open the existing Site source through the Sites source workflow helper with a fresh write credential. Copy the tracked `apps/web` source and LICENSE; exclude Git metadata, node_modules, dist, runtime state and credentials. Preserve `.openai/hosting.json`.
4. Build with the Sites build helper, push/package through the Sites source workflow helper, then save a version without deploying. Check desktop/mobile preview and all five analysis screens. Static SPA routes must also work on reload.
5. Confirm the owner is the sole allowed viewer, no editors/groups/external viewers. Deploy the exact saved version using the private deployment tool; wait for success. Verify the live app and access policy.
6. Record the URL, GitHub/source commit mapping and verification below. Create/review the GitHub PR and squash-merge with `[CF-Pages-Skip]`; confirm the Pages deployment remains unchanged.

## Verification record

Site: https://techvit-stats-workbench.steadymyna13.chatgpt.site

Project: `appgprj_6ae68f8fd5fc8191be03b23b829e4620`. Owner-only policy: one owner, no additional users, editors, groups or external viewers. Initial saved version 1 was deployed successfully on 2026-11-01. A final update with mobile layout fixes and expanded regression tests is being verified; its exact source SHA is recorded after deployment.

Executed successfully: `pnpm install --frozen-lockfile`, `pnpm test` (7 tests), `pnpm exec tsc --noEmit`, `pnpm build`, and `pnpm exec wrangler deploy --dry-run` (no bindings). Runtime: Node 24.22.1, pnpm 11.25.0 provided by the managed environment; repository pnpm pin 10.28.0 and lockfile are unchanged. No Rust code was changed; Rust workspace tests are not required for this Web-only change.

Known limits: real statistics, Rust execution, file/database access and TOML parsing remain unimplemented. Independent non-owner signed-in account testing and mobile viewport testing are not yet asserted.

## Official references

- https://help.openai.com/en/articles/20001339-creating-and-using-chatgpt-sites
- https://learn.chatgpt.com/docs/sites
- https://developers.cloudflare.com/pages/configuration/git-integration/github-integration/

Plus/Pro hosting is currently included, subject to shared limits. This app requests no database or file storage; building/editing uses Codex allowance. Recheck current plan terms when updating.
