# Cloudflare へのデプロイ(無料枠)

## 何をデプロイするか

| 対象 | デプロイ | 理由 |
|---|---|---|
| `crates/*`(CLI `sf`・Rust SDK) | **しない** | ローカルで Parquet / SQLite / PostgreSQL を読むコマンドラインツール。Cloudflare Workers は JS / WASM のみで、ファイルシステムも DB 直結もない。ユーザーは `cargo install` やバイナリ配布で使う |
| `apps/web`(Web ダッシュボード) | **Cloudflare Workers Static Assets** | Vite でビルドした静的 SPA。現状はモックデータで動くためバックエンド不要 |

## 料金: $0(Workers Free プラン)

- `apps/web/wrangler.jsonc` は `main` を持たない「静的アセットのみ」の Worker。
  静的アセット配信は Workers の課金対象外なので、Free プランで **月額 $0・リクエスト無制限**。
- 独自ドメインを使う場合も追加費用なし(ドメイン自体の年額のみ)。
- Cloudflare Pages でも同じく無料だが、Cloudflare は新規プロジェクトに Workers Static Assets を推奨しているのでこちらを採用した。

## 事前準備

1. Cloudflare アカウント(無料)
2. Node.js 22 以上・pnpm(`apps/web/package.json` の `packageManager` で固定)

## 手順 A: CLI から手動デプロイ

```bash
cd apps/web
pnpm install
pnpm exec wrangler login   # ブラウザで認証(初回のみ)
pnpm deploy:cf             # = pnpm build && wrangler deploy
```

`pnpm deploy` ではなく `pnpm deploy:cf` を使う(`pnpm deploy` は pnpm の組み込みコマンドで別物)。

初回デプロイで `https://statforge-dashboard.<あなたのsubdomain>.workers.dev` が発行される。

CI など非対話環境では `wrangler login` の代わりに `CLOUDFLARE_API_TOKEN`(Workers Scripts:Edit 権限)と `CLOUDFLARE_ACCOUNT_ID` を環境変数に設定する。

事前確認(アップロードせずにビルド内容だけ見る):

```bash
pnpm build && pnpm exec wrangler deploy --dry-run
```

## 手順 B: GitHub 連携で自動デプロイ(Workers Builds、無料)

Cloudflare ダッシュボード → Workers & Pages → Create → **Import a repository** で `sibukixxx/StatForge` を選び、以下を設定する。GitHub Actions は不要。

| 項目 | 値 |
|---|---|
| Root directory | `apps/web` |
| Build command | `pnpm install --frozen-lockfile && pnpm build` |
| Deploy command | `pnpm exec wrangler deploy` |
| Production branch | `main` |

`apps/web/package.json` の `packageManager` を見て pnpm が自動選択される(`preinstall` で npm を拒否しているため npm では失敗する)。
Workers Builds の無料枠は月 3,000 ビルド分・同時 1 ビルドで、この規模なら十分。

## ローカル確認

```bash
cd apps/web
pnpm build
pnpm exec wrangler dev   # http://localhost:8787 で本番同様の静的配信を確認
```

## 今後 CLI と連携させる場合

ダッシュボードから実データを解析したくなった場合の選択肢(いずれも現時点では未実装):

1. **sf-stats を WASM 化して Worker に載せる** — Polars 依存が重く、Worker の 1MB(Free)/10MB(Paid)制限に収めるのは困難
2. **Cloudflare Containers** で `sf` を動かす — Workers Paid($5/月〜)が必要
3. **ローカルで `sf ... --format json` を実行し、生成した JSON を静的アセットとして一緒にデプロイ** — $0 のまま実現でき、現実的

「最安値」を維持するなら 3 が推奨。
