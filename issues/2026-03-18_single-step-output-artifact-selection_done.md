# 単発CLIの出力対象artifactを明示選択する

- ステータス: done
- 進捗: 100%
- 最終更新: 2026-03-18

## 背景

- `crates/sf-cli/src/handlers/eda.rs` では単発 EDA コマンドに `json` と `parquet` を許可している
- `crates/sf-cli/src/handlers/common.rs` の `build_output_spec` は `artifacts` を空配列で渡している
- `crates/sf-report/src/lib.rs` は `artifacts` が空だと全 artifact を出力対象にする
- `crates/sf-report/src/parquet.rs` は最初に見つかった `Dataset` / `Table` artifact を Parquet に書き出す

## 対応結果

- 単発 CLI の file 出力時に、`OutputSpec.artifacts` へ生成 step の artifact 名だけを渡すように変更
- `sf-report` の Parquet writer で `Summary` / `Correlation` / `Distribution` / `OutlierDetection` の `Stats` artifact を表形式 `DataFrame` へ変換して書き出せるように変更
- CLI help と `README.md` の output format 説明を実装に合わせて更新
- `eda summary --format parquet` を実ファイル検証する integration test を追加
- `eda summary --format json` で `source` artifact が混ざらないことを確認する integration test を追加

## 検証結果

- `cargo run -p sf-cli -- eda summary --help`: pass
- `cargo run -p sf-cli -- stats outlier --help`: pass
- `cargo test -p sf-cli --test cli_eda`: pass
- `cargo test`: pass

## 完了条件

- 単発 CLI の `--out` が source ではなく生成成果物を対象にする
- `eda summary --format parquet` の期待挙動をテストで固定する
- help / README の出力フォーマット説明が実装と一致する

## 完了判定

- 上記 3 条件をすべて満たしたため完了
