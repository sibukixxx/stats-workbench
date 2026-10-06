# CLI共通化とRecipeバリデーション集約

- ステータス: done
- 進捗: 100%
- 最終更新: 2026-03-18

## 完了内容

- `sf-cli` の単発 EDA / Stats ハンドラで重複していた source 解決と output 構築を `crates/sf-cli/src/handlers/common.rs` に集約
- `SourceSpec::validate` と `StepSpec::validate` を `sf-core` に寄せて、CLI と recipe 実行の両方で同じ検証ルールを使う構成へ整理
- `PipelineEngine` で source ロードと context 初期化を `prepare_context` / `load_source` に共通化
- `StepSpec::kind_name` / `output_name` を導入して step 名と成果物名の決定ロジックを一元化
- CLI テストの fixture 準備を `crates/sf-cli/tests/support/mod.rs` に集約

## 確認結果

- `cargo fmt --check`: pass
- `cargo test`: pass

## 補足

- 既存の `issues` ディレクトリは存在しなかったため、このファイルは現時点の完了スナップショットとして追加
