# Stats Workbench 使い方ガイド

Stats Workbench (`stats-workbench`) は Parquet / SQLite / PostgreSQL のデータに対して、
探索的データ分析（EDA）や統計処理をコマンド一発で実行できる CLI ツールです。

## 目次

- [クイックスタート（3分で体験）](#クイックスタート3分で体験)
- [CLI コマンド一覧](#cli-コマンド一覧)
  - [stats-workbench eda summary — 要約統計](#sf-eda-summary--要約統計)
  - [stats-workbench eda corr — 相関分析](#sf-eda-corr--相関分析)
  - [stats-workbench eda dist — 分布確認](#sf-eda-dist--分布確認)
  - [stats-workbench stats outlier — 外れ値検出](#sf-stats-outlier--外れ値検出)
  - [stats-workbench run — レシピ実行](#sf-run--レシピ実行)
- [データソースの指定方法](#データソースの指定方法)
- [出力形式](#出力形式)
- [レシピ（TOML パイプライン）の書き方](#レシピtoml-パイプラインの書き方)
- [サンプルレシピ一覧](#サンプルレシピ一覧)
- [実践ユースケース](#実践ユースケース)

---

## クイックスタート（3分で体験）

### 1. ビルド

```bash
cargo build -p sf-cli
```

### 2. サンプルデータを生成

```bash
cargo run -p stats-workbench --example generate_sample_data
```

これで `examples/data/` に3つの Parquet ファイルが作られます:

| ファイル | 内容 | 行数 | 列 |
|---------|------|------|----|
| `sales.parquet` | 売上データ | 50 | date, product, category, price, quantity, revenue, discount, region |
| `employees.parquet` | 従業員データ | 30 | emp_id, department, years_exp, salary, performance_score, projects_completed |
| `sensors.parquet` | IoTセンサー | 100 | timestamp, sensor_id, temperature, humidity, pressure, battery_level |

### 3. 分析してみる

```bash
# 売上の基本統計
cargo run -p sf-cli -- eda summary \
  --source examples/data/sales.parquet \
  --cols price,quantity,revenue

# 給与と経験年数の相関
cargo run -p sf-cli -- eda corr \
  --source examples/data/employees.parquet \
  --cols years_exp,salary,performance_score \
  --method pearson

# 気温の分布
cargo run -p sf-cli -- eda dist \
  --source examples/data/sensors.parquet \
  --col temperature --bins 15

# 売上金額の外れ値検出
cargo run -p sf-cli -- stats outlier \
  --source examples/data/sales.parquet \
  --cols revenue

# レシピでまとめて実行
cargo run -p sf-cli -- run --recipe examples/05_full_pipeline.toml
```

> **ヒント**: `cargo install --path crates/sf-cli` でインストールすると `sf` コマンドとして直接使えます。

---

## CLI コマンド一覧

### stats-workbench eda summary — 要約統計

列ごとの件数、平均、標準偏差、最小・最大値などを一覧表示。

```bash
# 指定した列の統計
stats-workbench eda summary --source data.parquet --cols price,quantity,revenue

# 全列の統計（--cols を省略）
stats-workbench eda summary --source data.parquet
```

**出力例のイメージ:**

```
┌───────────┬───────┬────────┬───────┬───────┬────────┐
│ column    │ count │ mean   │ std   │ min   │ max    │
├───────────┼───────┼────────┼───────┼───────┼────────┤
│ price     │ 50    │ 98.50  │ 52.3  │ 10.0  │ 185.0  │
│ quantity  │ 50    │ 10.20  │ 5.8   │ 1.0   │ 20.0   │
│ revenue   │ 50    │ 1420.0 │ 890.2 │ 10.0  │ 3700.0 │
└───────────┴───────┴────────┴───────┴───────┴────────┘
```

### stats-workbench eda corr — 相関分析

複数列の相関係数を行列形式で出力。

```bash
# Pearson（線形相関、デフォルト）
stats-workbench eda corr --source data.parquet --cols x,y,z --method pearson

# Spearman（順位相関 — 非線形な関係も検出）
stats-workbench eda corr --source data.parquet --cols x,y,z --method spearman
```

**使い分け:**
- **Pearson**: 線形な関係を測る。正規分布に近いデータ向き
- **Spearman**: 単調な関係（線形でなくてもOK）を測る。外れ値に強い

### stats-workbench eda dist — 分布確認

指定した列のヒストグラム（度数分布）を計算。

```bash
# 10区間（デフォルト）
stats-workbench eda dist --source data.parquet --col temperature

# 区間数を指定
stats-workbench eda dist --source data.parquet --col temperature --bins 20
```

**ポイント:** `--col` は単数形（1列のみ指定）。`--cols` ではない点に注意。

### stats-workbench stats outlier — 外れ値検出

IQR（四分位範囲）法で外れ値候補を検出。

```bash
# 1列の外れ値
stats-workbench stats outlier --source data.parquet --cols revenue

# 複数列を同時にチェック
stats-workbench stats outlier --source data.parquet --cols revenue,price,quantity
```

**IQR 法**: Q1 - 1.5×IQR 未満、または Q3 + 1.5×IQR 超の値を外れ値と判定。

### stats-workbench run — レシピ実行

TOML ファイルに書いた分析手順をまとめて実行。

```bash
stats-workbench run --recipe examples/05_full_pipeline.toml
```

---

## データソースの指定方法

Stats Workbench は3種類のデータソースに対応しています。

### Parquet ファイル

```bash
stats-workbench eda summary --source path/to/data.parquet --cols col1,col2
```

### SQLite データベース

```bash
# テーブル指定
stats-workbench eda summary --source sqlite://path/to/database.db --table orders --cols amount,qty

# カスタム SQL クエリ
stats-workbench eda summary --source sqlite://path/to/database.db \
  --query "SELECT amount, qty FROM orders WHERE status = 'completed'" \
  --cols amount,qty
```

### PostgreSQL データベース

```bash
# テーブル指定
stats-workbench eda corr --source postgresql://user:pass@localhost:5432/mydb \
  --table metrics --cols latency,throughput

# 環境変数を使用（パスワードをコマンドに書かない）
export DATABASE_URL=postgresql://user:pass@localhost:5432/mydb
stats-workbench eda summary --source $DATABASE_URL --table users --cols age,score
```

レシピ内では `${DATABASE_URL}` で環境変数を展開できます。展開できるのは `DATABASE_URL` と `SF_` で始まる変数だけで、それ以外(`${HOME}` や `${AWS_SECRET_ACCESS_KEY}` など)を参照するレシピはエラーになります(共有レシピ経由の秘密情報の持ち出し防止):

```toml
[source]
type = "postgres"
url = "${DATABASE_URL}"
table = "metrics"
```

### --table と --query の使い分け

| オプション | 用途 |
|-----------|------|
| `--table orders` | テーブル全体を読み込む（簡単） |
| `--query "SELECT ..."` | フィルタや結合が必要な場合（柔軟） |

---

## 出力形式

### ターミナル表示（デフォルト）

何も指定しなければ、結果はターミナルにテーブル形式で表示されます。

### JSON ファイル出力

```bash
# CLI から
stats-workbench eda summary --source data.parquet --cols price,quantity \
  --out output/result.json --format json

# レシピで
[output]
format = "json"
path = "output/result.json"
artifacts = ["step1", "step2"]
```

### Parquet ファイル出力

```bash
stats-workbench eda summary --source data.parquet --cols price,quantity \
  --out output/result.parquet --format parquet
```

### ターミナル + ファイル同時出力（レシピのみ）

```toml
[output]
terminal = true           # 画面にも表示
format = "json"           # かつ JSON にも保存
path = "output/result.json"
artifacts = ["summary", "corr"]
```

---

## レシピ（TOML パイプライン）の書き方

レシピは「どのデータを」「どう分析して」「どこに出すか」を TOML で宣言するファイルです。

### 基本構造

```toml
# 1. データソース
[source]
type = "parquet"              # parquet / sqlite / postgres
path = "data/sample.parquet"  # ファイルパスまたは URL

# 2. 分析ステップ（複数書ける）
[[steps]]
id = "step_name"              # ステップの名前（出力で参照）
type = "eda.summary"          # 分析の種類
cols = ["col1", "col2"]       # 対象列

# 3. 出力設定（省略可）
[output]
terminal = true
format = "json"
path = "output/result.json"
artifacts = ["step_name"]     # 出力するステップを指定
```

### ステップの種類

| type | 説明 | 必須パラメータ |
|------|------|---------------|
| `eda.summary` | 要約統計 | `cols` (省略で全列) |
| `eda.correlation` | 相関分析 | `cols` (2列以上), `method` |
| `eda.dist` | 分布 | `col` (1列), `bins` (省略時10) |
| `stats.outlier_detect` | 外れ値検出 | `cols`, `method = "iqr"`, `action = "flag"` |

### ステップ間のデータの流れ

- 各ステップは独立して実行される（前のステップの結果に依存しない）
- すべてのステップは同じデータソースを参照する
- `id` を付けておくと、`[output].artifacts` で出力するステップを選べる

### レシピの実行

```bash
stats-workbench run --recipe path/to/recipe.toml
```

---

## サンプルレシピ一覧

すぐに試せるサンプル（`cargo run --example generate_sample_data` でデータ生成後）:

| ファイル | 内容 | データ |
|---------|------|--------|
| [`01_basic_summary.toml`](01_basic_summary.toml) | 要約統計の基本 | sales |
| [`02_correlation.toml`](02_correlation.toml) | Pearson / Spearman 相関比較 | employees |
| [`03_distribution.toml`](03_distribution.toml) | ヒストグラムの確認 | sensors |
| [`04_outlier_detection.toml`](04_outlier_detection.toml) | IQR 外れ値検出 | sales |
| [`05_full_pipeline.toml`](05_full_pipeline.toml) | 要約→相関→分布→外れ値の一括実行 | sales |
| [`06_json_output.toml`](06_json_output.toml) | JSON 出力の例 | sensors |
| [`07_employee_analysis.toml`](07_employee_analysis.toml) | 実践: 人事データ分析 | employees |

実データを使ったサンプル（別途ダウンロードが必要）:

| ディレクトリ | データセット | 内容 | 見どころ |
|-------------|-------------|------|----------|
| [`bike_sharing/`](bike_sharing/README.md) | UCI Bike Sharing | 気象条件と自転車レンタル需要の関係 | 気温↔需要の中程度相関 |
| [`nyc_taxi/`](nyc_taxi/README.md) | NYC Taxi | タクシー運賃と乗車距離の分析 | 大規模データの分布分析 |
| [`noaa_weather/`](noaa_weather/README.md) | NOAA Climate | 気温・降水量の相関分析 | 時系列データの相関 |
| [`titanic/`](titanic/README.md) | Titanic 乗客 | 運賃と生存率の相関分析 | 運賃の右裾分布、外れ値 |
| [`iris/`](iris/README.md) | Iris（アヤメ） | 花の形態計測値の相関 | **r≈0.96 の超強相関**、二峰性分布 |
| [`auto_mpg/`](auto_mpg/README.md) | Auto MPG（燃費） | 車両特性と燃費の関係 | **r≈-0.83 の強い負の相関** |
| [`wine_quality/`](wine_quality/README.md) | Wine Quality（ワイン品質） | 化学成分と品質スコア | 中程度相関、化学成分の外れ値 |
| [`heart_disease/`](heart_disease/README.md) | Heart Disease（心臓病） | 健康リスク要因の相関 | 臨床データの外れ値の意味 |
| [`diamonds/`](diamonds/README.md) | Diamonds（ダイヤモンド） | カラット数と価格の関係 | **r≈0.92 の超強相関**、極端な右裾 |
| [`student_performance/`](student_performance/README.md) | Student Performance（学生成績） | 成績の経時変化と欠席 | **r≈0.80**、二峰性分布 |
| [`california_housing/`](california_housing/README.md) | California Housing（住宅価格） | 所得と住宅価格の関係 | 天井効果、居住密度の外れ値 |
| [`spotify_tracks/`](spotify_tracks/README.md) | Spotify Tracks（楽曲） | 音響特徴量の相関 | **r≈0.76**、人気度のべき分布 |
| [`fifa_players/`](fifa_players/README.md) | FIFA 22 Players（選手） | 選手能力値と年俸 | 年俸のスーパースター外れ値 |

---

## 実践ユースケース

### ユースケース1: 売上データの探索

「新しいデータセットを受け取った。まず何を見ればいい？」

```bash
# 1. まず全体像を把握
stats-workbench eda summary --source data/sales.parquet

# 2. 数値列の相関を見て、関係の強い変数を特定
stats-workbench eda corr --source data/sales.parquet --cols price,quantity,revenue --method pearson

# 3. 売上金額の分布を確認（正規分布？偏りがある？）
stats-workbench eda dist --source data/sales.parquet --col revenue --bins 20

# 4. 外れ値を検出（異常な取引がないか）
stats-workbench stats outlier --source data/sales.parquet --cols revenue,price
```

### ユースケース2: データクリーニングの前段

「分析前に、データの品質を確認したい」

```bash
# 欠損値の有無を count で確認（count < 全件数なら欠損あり）
stats-workbench eda summary --source data/raw.parquet --cols col1,col2,col3

# 外れ値候補を確認
stats-workbench stats outlier --source data/raw.parquet --cols col1,col2
```

### ユースケース3: DB から直接分析

「アプリの DB にあるデータを、エクスポートせずにそのまま分析したい」

```bash
# SQLite の場合
stats-workbench eda summary --source sqlite://app.db --table users --cols age,purchase_count

# 条件付きで分析したい場合は --query を使う
stats-workbench eda corr --source sqlite://app.db \
  --query "SELECT age, purchase_count, total_spent FROM users WHERE active = 1" \
  --cols age,purchase_count,total_spent
```

### ユースケース4: 分析結果をチームで共有

「分析結果を JSON で保存して、ダッシュボードや別のスクリプトから参照したい」

```bash
# レシピを書いて再現可能にする
stats-workbench run --recipe analysis/quarterly_report.toml

# output/ に JSON が出力される → git で管理、CI で自動実行も可能
```

### ユースケース5: Pearson と Spearman の使い分け

```bash
# まず Pearson で線形相関を確認
stats-workbench eda corr --source data.parquet --cols x,y --method pearson

# 相関が低いが関係がありそうなら Spearman で非線形を確認
stats-workbench eda corr --source data.parquet --cols x,y --method spearman

# Spearman のほうが高ければ、非線形な単調関係がある
```

---

## よくある質問

### Q: `--cols` を省略するとどうなる？

`eda summary` では全列が対象になります。`eda corr` と `stats outlier` では `--cols` は必須です。

### Q: `--col` と `--cols` の違いは？

- `--cols`: カンマ区切りで複数列を指定（summary, corr, outlier）
- `--col`: 1列だけ指定（dist）

### Q: レシピの `id` は何に使う？

`[output].artifacts` で、どのステップの結果を出力するか指定するときに使います。
`id` を省略すると自動生成されます（`step_01_eda_summary` のような名前）。

### Q: 大きなファイルでも動く？

Parquet は LazyFrame（遅延評価）で読み込まれるため、メモリに全データを載せずに処理できます。

---

## 統計結果の解釈ガイド

### 相関係数の読み方

| 相関係数 |r| | 強さ | 実データでの例 |
|----------|------|------|
| 0.9 〜 1.0 | 非常に強い | iris: petal_length ↔ petal_width (0.96), diamonds: carat ↔ price (0.92) |
| 0.7 〜 0.9 | 強い | auto_mpg: weight ↔ mpg (-0.83), student: G1 ↔ G3 (0.80) |
| 0.4 〜 0.7 | 中程度 | california_housing: MedInc ↔ MedHouseVal (0.69), wine: alcohol ↔ quality (0.48) |
| 0.2 〜 0.4 | 弱い | titanic: Pclass ↔ Survived (-0.34) |
| 0.0 〜 0.2 | ほぼなし | 実用的な相関とは言えない |

### Pearson と Spearman の比較

- **Pearson ≈ Spearman**: 線形関係が支配的
- **Spearman > Pearson**: 非線形だが単調な関係がある（例: 指数関数的な価格上昇）
- **Pearson > Spearman**: まれだが、外れ値の影響を受けている可能性

### 外れ値検出（IQR法）の解釈

- **外れ値 < 5%**: 正規分布に近い、少数の異常値
- **外れ値 5〜15%**: 分布に裾が長い（右裾分布など）
- **外れ値 > 15%**: 非正規分布、複数の母集団が混在している可能性

### 分布の形状

- **左右対称**: 正規分布に近い（例: auto_mpg の加速性能）
- **右裾（正の歪み）**: 少数の高値が存在（例: diamonds の価格、fifa の年俸）
- **二峰性**: 2つの異なるグループが混在（例: iris の花弁長、student の最終成績）
- **天井効果**: データに上限がある（例: california_housing の住宅価格 $500K キャップ）
