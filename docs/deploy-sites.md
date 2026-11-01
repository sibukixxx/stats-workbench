# ChatGPT Sites

## Scope

TechVit Stats Workbench uses the existing `apps/web` Preact UI as a static demo.
The Rust CLI, SDK and statistical algorithms remain unchanged. `src/state.ts`
calls the sample result builders in `src/data.ts`; no real analysis engine,
file upload, database connection or persistent user data is implemented.
Do not enter customer data, passwords or actual connection strings.

## Configuration

- Project: `TechVit Stats Workbench`
- Deployed URL: https://techvit-stats-workbench.steadymyna13.chatgpt.site
- Audience: owner only, enforced by Sites; no application auth is added.
- Source manifest: `apps/web/.openai/hosting.json`, static directory `dist`.
- Node.js 24.22.1 and pnpm 11.25.0 (managed runner); the existing
  `packageManager: pnpm@10.28.0`, lockfile and Vite 6.4.5 are retained.
- D1, R2, connectors and runtime secrets: none.
- Cost: no additional database/file resources; Sites hosting is subject to the
  account's usage limits and pricing, not a guaranteed unlimited free service.

Sites accepts prebuilt static assets, so no Vite major upgrade or Vinext
migration is needed. The existing Cloudflare build remains `pnpm build`.

## Update procedure

1. Update `apps/web` on a GitHub branch; keep the existing engine boundary.
2. In `apps/web`, run `pnpm install --frozen-lockfile`, `pnpm test`,
   `pnpm exec tsc --noEmit`, and `pnpm build`.
3. Copy the tracked `apps/web` source to the same Sites checkout, excluding
   `.git`, `node_modules`, `dist`, local environment files and runtime state.
   Preserve the project ID; do not create another Site.
4. Use the Sites source workflow helper to build, push and package that source.
   Save a version without deployment, inspect the local preview, then deploy
   that saved version with the owner-private operation.
5. Check the returned deployment status and access policy; test the live UI.
6. Record the GitHub source commit, Sites source commit and verification results.

## Cloudflare coexistence

The existing Pages project `techvit-stats-private` serves `stats.techvit.me`
and is connected to `sibukixxx/stats-workbench`, branch `main`, root `apps/web`.
Its build command is `corepack enable && pnpm install --frozen-lockfile && pnpm build`
and output is `dist`. Main merges trigger its normal automatic build.
No Pages settings, Access policies, Secrets, domains or databases are changed.
The older `deploy-cloudflare.md` describes an alternative Workers deployment;
it does not describe the currently running Pages project's GitHub integration.

## Verification

2026-11-01: saved version 1 without deployment, then deployed successfully
using the owner-private operation. Sites source commit:
`97ee1872bafd3d9aca86f39d14319c505f508698` (the `apps/web` source snapshot).

| Check | Result |
| --- | --- |
| `pnpm install --frozen-lockfile` | Passed, lockfile unchanged |
| `pnpm test` | 9 passed; navigation, summary/correlation/distribution/outlier/recipe mock execution, validation and demo notice |
| `pnpm exec tsc --noEmit` | Passed |
| `pnpm build` | Passed; existing Vite 6 static build |
| `pnpm exec wrangler deploy --dry-run` | Passed; no bindings; no Cloudflare deployment performed |
| Owner-only access policy | Verified: one owner, zero external visitors, zero groups |
| Anonymous live browser | Login gate shown; application content not exposed |
| PC preview and interactions | Rechecked successfully after preview became available: all five mock executions, navigation, distribution bin change and `/run` reload |
| Mobile visual QA and authenticated live operations | Unverified: no mobile viewport control in the available browser API; live cloud browser has no owner login |
| Different signed-in user's access denial | Not exercised; owner-only policy and private-deploy backend check passed |

No real statistical computation is implemented in the web UI. The existing
Cloudflare documentation describes Workers while the running deployment uses
Pages; use the coexistence section above for the observed current settings.

Independent follow-up verification used GitHub main `5532bad5882573c8268e49824433d070d4d44d05`.
Its `apps/web/src` matches deployed Sites source `97ee1872bafd3d9aca86f39d14319c505f508698`
byte for byte. Frozen installation, all nine tests, build, explicit TypeScript
check and Cloudflare dry-run passed again. This documentation-only follow-up
does not require a new Sites deployment.

Official references consulted:
- https://help.openai.com/en/articles/20001339-creating-and-using-chatgpt-sites
- Sites plugin's static-hosting, preview and identity documentation.

## Preview and responsive follow-up

The managed preview host `terminal.local` is explicitly allowed. Node.js 22+
is declared, with `pnpm typecheck` as the explicit TypeScript check. Long labels
wrap and the correlation matrix scrolls horizontally on narrow screens.
The local browser displayed the correlation controls and matrix in a 390px
iframe viewport and executed the mock successfully. This is responsive-layout
verification, not a physical-phone or authenticated production test.
Nine tests, typecheck, build, frozen install and Cloudflare dry-run passed.
No dependencies, Rust crates, Cloudflare settings, or storage bindings changed.
