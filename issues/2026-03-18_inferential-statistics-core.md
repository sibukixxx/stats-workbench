# 推測統計コアを導入する

- ステータス: open
- 優先度: highest
- 進捗: 企画
- 最終更新: 2026-03-18

## 背景

- 現在の StatForge は `summary` / `corr` / `dist` / `outlier` までで、記述統計と EDA が中心
- ユーザーが挙げたトピックのうち、プロ向け統計システムとして最も大きい欠落は「推定」と「仮説検定」
- `StatsKind` には `TTest` / `Anova` がある一方で、`StepSpec` / CLI / executor / report には未接続

## この課題でカバーする領域

- 確率分布の実務利用
  - t分布
  - カイ二乗分布
  - F分布
- 推定
  - 母平均の区間推定
  - 母比率の区間推定
  - 母分散の区間推定
- 仮説検定
  - 1標本 t 検定
  - 2標本 t 検定
  - 母比率の検定
  - 母分散の検定
  - カイ二乗適合度検定
  - カイ二乗独立性検定
- 検定結果の基礎情報
  - p値
  - 信頼区間
  - 検定統計量
  - 自由度
  - 効果量

## システム変更の要点

- `sf-core`
  - `StepSpec` に infer 系 step を追加
  - `StatsKind` に `ChiSquareTest` / `ConfidenceInterval` など不足種別を追加
  - 推定・検定結果の JSON schema を定義
- `sf-stats`
  - `infer` モジュールを新設
  - 分布関数、p値計算、区間推定ロジックを実装
- `sf-engine`
  - infer step の実行分岐を追加
- `sf-cli`
  - `infer` コマンド群を追加
  - 例: `sf infer mean-ci`, `sf infer ttest-1`, `sf infer chisq-independence`
- `sf-report`
  - terminal / JSON / Parquet の出力整形を追加

## スコープ上の判断

- このフェーズでは「理論統計の網羅」ではなく「現場でまず使える推測統計 API」を優先する
- 第1種・第2種の過誤や検出力は説明出力には含めるが、power analysis の本実装は次フェーズに分離してよい
- 観察研究・実験研究・標本調査法そのものは分析実行エンジンとは別の設計面が大きいため、この課題には含めない

## 完了条件

- CLI と recipe の両方から推定・検定 step を実行できる
- 1標本 / 2標本 / 比率 / 分散 / カイ二乗系の最小回帰テストが揃う
- terminal / JSON / Parquet で推定・検定結果を読める
- p値、信頼区間、自由度、効果量を最低限一貫して出力する

## 次の一手

- 最初の着手は `1標本 t 検定` と `母平均の区間推定`
- この2つで分布、p値、自由度、信頼区間、report schema の土台を固める
