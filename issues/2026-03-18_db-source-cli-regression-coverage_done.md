# DBソース系CLIの回帰テストを追加する

- ステータス: done
- 進捗: 100%
- 最終更新: 2026-03-18

## 背景

- 現在の CLI 統合テストは Parquet fixture 中心で、`sqlite://` / `postgresql://` 系の経路を通していない
- `SourceSpec::validate` の責務を `sf-core` に寄せたため、DB source の検証経路は今後 regress しやすい
- `SourceLoader::parse_cli_source` と `PipelineEngine::execute_single` の境界を CLI 側から守るテストが不足している

## 対応結果

- `crates/sf-cli/tests/support/mod.rs` に一時 SQLite fixture を生成する helper を追加
- `crates/sf-cli/tests/cli_db.rs` を新設し、以下を追加
  - SQLite source を `--table` で読む正常系
  - SQLite source を `--query` で読む正常系
  - SQLite source で `--table` / `--query` が無いときの validation エラー
  - PostgreSQL source で `--table` / `--query` が無いときの validation エラー
- `crates/sf-data/src/source.rs` に `parse_cli_source` の unit test を追加し、`pg://` が `postgresql://` へ正規化されることを固定
- `crates/sf-core/src/recipe.rs` の `SourceSpec::validate` test を強化し、DB source validation の文言を固定

## 検証結果

- `cargo test -p sf-cli --test cli_db`: pass
- `cargo test -p sf-data`: pass
- `cargo run -p sf-cli -- eda summary --source pg://localhost:5432/app --cols price`: expected validation error を確認
- `cargo test`: pass

## 完了条件

- SQLite 正常系が少なくとも 2 ケース追加されている
- DB source の validation エラーが CLI 層まで期待どおりに伝播する
- source 種別ごとの最小回帰テストセットが `sf-cli/tests` に揃う

## 完了判定

- 上記 3 条件をすべて満たしたため完了
