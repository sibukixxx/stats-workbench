# OSS migration verification — stats-workbench

検証日: 2026-10-26。実装コードの検証対象head: `0a2217f082343deb4d46ff3307927d6ddaeebd30`。本書の追記commitは資料のみ。

## 実装と実行結果

- Rust 1.99.1: workspace test/build、fmt --check、clippy --workspace --all-targets -- -D warnings: 成功。
- Python exporter 6件と人口fixture 1件、Web 5件・build・pnpm audit: 成功。
- 実stats-workbench binary→Insight main 6885c6005c091b70cc0e72f705e103cc6482ca28からbuildした実delivery binaryのloopback HTTP E2E: 成功（skipなし）。STATS_WORKBENCH_COMMIT=0a2217f082343deb4d46ff3307927d6ddaeebd30で再実行。
- 7 observations、Findingsなし、idempotent receipt、再実行fingerprint一致を確認。fingerprint=529f9bce49a9a0261d01ce44653a900abcf933c4d14b3d0e08f963d615418162。
- cargo audit（RustSec DB 21d909e4c8bd4e14c7aea58bf980c881a0053ab1、2026-10-24更新）: 既知脆弱性0件。間接proc-macro依存paste 1.0.15にRUSTSEC-2024-0436（保守終了）の情報警告1件。隠蔽・ignore指定なし。
- `cargo build -p sf-cli --release --locked` と最適化binaryのQuick Start/実Insight E2E: 成功。
- Rust 1.89の最低版実行、DB外部接続、live統計ソース再取得は未実施。

## 公開前監査

- GitHub APIで全refs（main、作業branch、残存branch、PR head参照）と到達可能なcommitのtreeを列挙。列挙時点で122 commits / 330 unique blobsを取得、欠落・truncated treeなし。以降の当作業commitは現行tree検査と併せて確認。
- Gitleaks既定ruleで履歴の全blobとcommit metadataを検査。secret検出なし。
- PR最終treeは全fileを用意してGitleaks成功。localの検証対象sourceとGitHub PR headのblob hash一致を確認。
- 全履歴のauthorはYuichi Takada、メールは866555+sibukixxx@users.noreply.github.com。LICENSEはMIT。
- CI startup_failureはテスト実行前の起動失敗。CI成功とは記録せず、上記local gateを代替証拠とする。既存CIは削除していない。

## 未完了の公開操作

4正本はPrivate。Public化・Release作成・匿名clone/downloadは未実施。GitHub connectorはadministration/Release作成を提供せず、ブラウザは未ログイン。移行公開完了とは扱わない。公開後に匿名repository/README/LICENSE/Release/Quick Startを再検証して公式サイトをAvailableへ変更する。

予定初回tag: v0.1.0。旧repoは削除・archiveせず、共有履歴のforce pushなし。未完了Issueを閉じない。DNS・メール・他Workers/Pages・顧客環境は変更していない。
