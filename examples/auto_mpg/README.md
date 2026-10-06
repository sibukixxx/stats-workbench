# Example: Auto MPG（自動車燃費）データセット

## データソース

- **出典**: UCI Machine Learning Repository / CMU StatLib
- **ライセンス**: Public domain
- **URL**: https://archive.ics.uci.edu/ml/datasets/auto+mpg

## データ概要

- 約392台の車両データ（欠損値除去後）
- 7つの数値特徴量: mpg, cylinders, displacement, horsepower, weight, acceleration, model_year

## セットアップ

```bash
# データのダウンロード
./download.sh

# Parquet に変換（pandas + pyarrow が必要）
python3 prepare.py
```

## レシピ一覧

| ファイル | 分析内容 |
|----------|----------|
| `corr_efficiency.toml` | 燃費効率の相関分析（Pearson / Spearman） |
| `dist_mpg.toml` | 燃費の分布分析 + 基本統計量 |
| `outlier_weight.toml` | 重量・馬力・排気量の外れ値検出 |

## 実行例

```bash
# 相関分析
stats-workbench run examples/auto_mpg/corr_efficiency.toml

# 分布分析
stats-workbench run examples/auto_mpg/dist_mpg.toml

# 外れ値検出
stats-workbench run examples/auto_mpg/outlier_weight.toml
```

## このデータセットで示せること

- **重量と燃費の強い負の相関** (weight - mpg: r=-0.83) -- 物理法則の反映
- **排気量・馬力も燃費と強い負の相関** (displacement - mpg: r=-0.80, horsepower - mpg: r=-0.78)
- 加速性能の分布、重量の外れ値検出

## 期待される知見

- **weight - mpg: r=-0.83** は「強い負の相関」-- 重い車ほど燃費が悪い
- **displacement - mpg: r=-0.80** も同様に強い負の相関
- **horsepower - mpg: r=-0.78**
- これらの負の相関は物理的に自明だが、定量化することで「どの要因が最も影響するか」がわかる
- Pearson と Spearman の差が小さければ線形関係が支配的

## 結果の解釈

1. **最も影響力の大きい要因**: weight（重量）が燃費に対して最も強い負の相関を示す。車両設計において軽量化が燃費改善の最優先事項であることを裏付ける
2. **多重共線性**: weight, displacement, horsepower は互いに強い正の相関を持つ。これらは独立した要因ではなく、「大きいエンジン = 重い車 = 高馬力」という構造的な関係がある
3. **加速性能**: acceleration は他の変数との相関が比較的弱く、設計上のトレードオフが多様であることを示唆する
4. **外れ値**: 極端に重い車や高馬力車は、特定のセグメント（大型車・スポーツカー）に対応する。これらを除外して分析するか含めるかで結果が変わりうる
