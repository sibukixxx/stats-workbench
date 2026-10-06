# 時系列分析の基盤を導入する

- ステータス: done
- 優先度: high
- 進捗: 完了
- 最終更新: 2026-03-18

## 背景

- ユーザーが挙げた時系列処理、確率過程、系列相関、ADF、分解、ARIMA は現状システム化されていない
- `StatsKind` には `Adf` / `Decompose` / `ArimaForecast` があるが、実行導線は未実装
- 既存 examples に時系列性の強いデータがあり、実利用との相性がよい

## この課題でカバーする領域

- 時系列データの前処理
  - 時刻列の指定
  - ソート検証
  - 欠損・重複時点の扱い
  - lag / diff / rolling 集計
- 定常性確認
  - ADF 検定
- 分解
  - trend / seasonal / residual
- 予測
  - ARIMA の最小実装
  - 予測区間
- 系列相関の初期診断
  - 自己相関の要約

## システム変更の要点

- `sf-core`
  - 時系列 step spec を追加
  - forecast artifact と decomposition artifact の schema を明確化
- `sf-stats`
  - `timeseries` モジュールを新設
  - ADF、分解、ARIMA、lag/diff/rolling を実装
- `sf-engine`
  - 時系列 step 実行を追加
- `sf-cli`
  - 例: `sf ts adf`, `sf ts decompose`, `sf ts forecast`
- `sf-report`
  - forecast table、信頼区間、trend/seasonal 出力を追加

## スコープ上の判断

- 第1フェーズは ARIMA と ADF と分解まで
- マルコフ連鎖、ランダムウォーク、ポアソン過程、ブラウン運動などの高度確率過程は別課題に分離

## 完了条件

- ADF / 分解 / ARIMA forecast を CLI / recipe から実行できる
- 時刻列と頻度の validation がある
- forecast artifact を terminal / JSON / Parquet で出せる
- 時系列用の fixture と integration test が揃う

## 次の一手

- 最初の着手は `ADF 検定` と `lag/diff/rolling`
- ここで時系列列の扱いと report schema を固めてから分解と予測へ進む
