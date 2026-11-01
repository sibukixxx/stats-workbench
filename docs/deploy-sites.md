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
and output is `dist`. Main merges normally trigger its automatic build. For Sites-only changes, prefix the squash merge commit with `[CF-Pages-Skip]` to keep the existing Cloudflare deployment unchanged.
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

## Final update (2026-11-01)

Saved version 2 before publication, then successfully deployed using the owner-private operation.
Sites source commit: `dca0a81c18716517235c3e21274922ef687c81f8`.
The application snapshot is recorded by the responsive/verification follow-up PR. This version retains all existing Preact screens and Vite 6, adds explicit demo disclosure, a Node.js 22+ declaration and typecheck script, allows only terminal.local for managed development preview, and fixes narrow-screen overflow and sidebar scrolling.

Validation: frozen install, TypeScript, production build and Wrangler dry-run passed. Seven UI tests passed before publication; reconciliation preserves the nine tests already on main, all passing again. Desktop preview: all five executions passed. Overview was visually checked in a 390px iframe viewport (375px content area), with no page-level horizontal overflow; physical smartphone and full mobile workflow remain unverified. The anonymous production browser displays the ChatGPT login gate; authenticated production operations and a second signed-in user's rejection remain unverified. Access policy was rechecked: sole owner, no editors/groups/external visitors.

No D1/R2, new credentials, database or application authentication resources were created. Plus/Pro hosting is currently included subject to usage limits; editing/building consumes Codex allowance.

Concurrent main merges #11 and #14 triggered normal Cloudflare Pages builds during this work. No Cloudflare settings were changed. This follow-up is squash-merged with `[CF-Pages-Skip]` to avoid another Pages deployment. The initial Pages production source was a17b67c; it should not be reported as unchanged after those concurrent merges.

Additional documentation mismatch: deploy-cloudflare.md describes obtaining a workers.dev address, while wrangler.jsonc has workers_dev disabled. The running Pages project and its domains are separate from that alternative Workers configuration.

## Correlation matrix follow-up

The correlation matrix now scrolls horizontally within its card on narrow
screens instead of clipping the remaining columns. Its controls, result matrix
and mock execution were verified in a 390px iframe viewport. This extends the
overview responsive check above; physical-phone and authenticated production
tests remain unverified. The existing engine and deployment settings are unchanged.

## Version 3 release (2026-11-01 10:14 JST)

Saved version 3 without deployment, checked the local preview, then deployed it
successfully with the owner-private operation.

- GitHub application source: `d5f0e2e5bb8194afb9535bd3b3c14f06dac934ef` (PR #15).
- Sites source: `60a52d771722138b31ee383e415f5518f1cb1bee`.
- URL: https://techvit-stats-workbench.steadymyna13.chatgpt.site
- The Sites repository records GitHub provenance in `.openai/github-source.json`.
- Final checks passed: frozen install, 9 UI tests, typecheck, Vite build, and
  Wrangler dry-run (zero bindings).
- Browser preview passed: overview, all five analysis/recipe executions,
  navigation, distribution bins changed from 8 to 5, and direct `/run` loading.
  Narrow-screen correlation controls/matrix and execution were checked in a
  390px iframe. This does not substitute for physical-smartphone testing.
- After publication, owner-only access was rechecked: one owner, zero editors,
  external visitors, or groups. Anonymous production access shows the login gate.
- Authenticated production UI operation and denial of a different signed-in
  account remain unverified because this browser has no owner login.

This final documentation update does not change the deployed application source.
