# ChatGPT Sites deployment

## Scope

The existing Preact/Vite 6 SPA is packaged as static assets. No Vinext migration, runtime server, D1, R2, secrets, or application authentication is needed. Sites sharing enforces owner-only access. Rust CLI/SDK and algorithms are unchanged. `src/state.ts` remains the UI execution boundary for a future engine integration; current results come from `src/data.ts` synthetic fixtures.

## Build and update

Use Node.js 22+ (validated on 24.22.1) and the pnpm version in `apps/web/package.json`.

```sh
cd apps/web
pnpm install --frozen-lockfile
pnpm test
pnpm exec tsc --noEmit
pnpm build
pnpm exec wrangler deploy --dry-run
```

`apps/web/.openai/hosting.json` retains the Sites identity and declares `dist` as static output. Copy only the web source (excluding Git metadata, node_modules and dist) into the Site source checkout, build with the Sites helper, push and package that exact source commit. Save a version without deployment, preview it, verify owner-only sharing, then deploy that saved version using the private deployment operation. Never publish publicly. Do not deploy Cloudflare as part of this workflow.

The existing `stats.techvit.me` Pages project remains unchanged. Read-only inspection on 2026-11-01 confirmed GitHub `sibukixxx/stats-workbench`, main branch, root `apps/web`, output `dist`, production automatic deployments enabled, previews disabled. Merging web changes can therefore trigger its existing automatic build.

## Limits

Demo only: source paths, database URL, SQL and recipe inputs are UI examples; they are not sent to a server or executed. Do not enter customer data or credentials. No persistence. Deep links require SPA fallback; verify deployed routes before claiming support.

## Verification and publication

- Site: https://techvit-stats-workbench.steadymyna13.chatgpt.site
- Project: `appgprj_6ae68f8fd5fc8191be03b23b829e4620`
- Deployed Sites source: `97ee1872bafd3d9aca86f39d14319c505f508698` (web sources match this change; Sites source is a separate Git repository).
- Saved version 1: `appgprj_6ae68f8fd5fc8191be03b23b829e4620~appgver_4811fbd32e848191aee832b8c83c68c1`.
- Native deployment status: succeeded on 2026-11-01. This version was saved and deployed concurrently by another session; this session fetched and compared the deployed web source before adopting it. Its demo wording and execution assertions were adopted into this GitHub change; Sites also carries checkout-local package-manager policy.
- Frozen install, 9 UI tests, build, standalone TypeScript check and Cloudflare Wrangler dry-run passed. Vitest covers navigation, summary execution, correlation execution, distribution execution, outlier execution, recipe execution and source validation.
- Access policy: custom; one owner; no external viewers, groups or editors. Anonymous browser displays the sign-in gate. A separate signed-in non-owner account was not available for an end-to-end denial test.
- Preview server became healthy after allowing the exact managed preview hostname, but browser URL policy blocked local preview inspection. PC/mobile visual QA, signed-in production UI interactions and deep-link reload remain unverified. No viewport emulation API was available. Do not infer these checks from unit tests.
- Static assets only (about 76 KB uncompressed); no D1/R2 or other new Cloudflare resources.

Existing Cloudflare documentation describes Workers Static Assets, while the existing live `techvit-stats-private` project is Pages with GitHub integration. Its current Pages settings were inspected without modification; the Workers configuration remains an optional supported build target.

## Official references

- https://help.openai.com/en/articles/20001339-creating-and-using-chatgpt-sites
- Sites installed workflow: static output supports an existing framework build; no forced Vite major upgrade.

Hosting is currently included with Plus/Pro, subject to shared usage limits. No additional Cloudflare resources are provisioned.
