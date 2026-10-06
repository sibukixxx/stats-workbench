# Example: Wine Quality（ワイン品質）データセット

## データセット

- **出典**: [UCI Machine Learning Repository — Wine Quality](https://archive.ics.uci.edu/ml/datasets/wine+quality)
- **参考文献**: P. Cortez, A. Cerdeira, F. Almeida, T. Matos and J. Reis (2009). *Modeling wine preferences by data mining from physicochemical properties.*
- **レコード数**: 1599（赤ワインのみ）
- **カラム数**: 12（11の物理化学的特性 + 品質スコア 0-10）

### カラム一覧

| カラム | 説明 |
|--------|------|
| `fixed_acidity` | 固定酸度（酒石酸 g/dm³） |
| `volatile_acidity` | 揮発酸度（酢酸 g/dm³） |
| `citric_acid` | クエン酸（g/dm³） |
| `residual_sugar` | 残糖（g/dm³） |
| `chlorides` | 塩化物（塩化ナトリウム g/dm³） |
| `free_sulfur_dioxide` | 遊離二酸化硫黄（mg/dm³） |
| `total_sulfur_dioxide` | 総二酸化硫黄（mg/dm³） |
| `density` | 密度（g/cm³） |
| `pH` | pH値 |
| `sulphates` | 硫酸塩（硫酸カリウム g/dm³） |
| `alcohol` | アルコール度数（%） |
| `quality` | 品質スコア（0-10、官能評価の中央値） |

## このサンプルで確認できること

- **相関分析**: alcohol と quality の中程度の正の相関（r ≈ 0.48）、volatile_acidity と quality の負の相関（r ≈ -0.39）
- **分布分析**: 品質スコアは 5-6 に集中する偏った分布
- **外れ値検出**: residual_sugar、chlorides、total_sulfur_dioxide に極端な値が存在

## データ準備

```bash
cd examples/wine_quality
./download.sh
python3 prepare.py
```

## 実行

```bash
# 相関分析: アルコール・化学成分と品質の関係
stats-workbench run --recipe examples/wine_quality/corr_quality.toml

# 分布分析: 品質スコアの分布
stats-workbench run --recipe examples/wine_quality/dist_quality.toml

# 外れ値検出: 化学成分の異常値
stats-workbench run --recipe examples/wine_quality/outlier_chemistry.toml

# CLI で直接実行する場合:
stats-workbench eda corr --source examples/wine_quality/data/wine_quality.parquet --cols alcohol,volatile_acidity,quality --method pearson
stats-workbench eda dist --source examples/wine_quality/data/wine_quality.parquet --col quality --bins 15
stats-workbench stats outlier --source examples/wine_quality/data/wine_quality.parquet --cols residual_sugar,chlorides
```

## 結果の解釈

### 相関の強さの目安

| 相関係数 (|r|) | 解釈 |
|----------------|------|
| 0.00 - 0.19 | ほぼ無相関 |
| 0.20 - 0.39 | 弱い相関 |
| 0.40 - 0.59 | 中程度の相関 |
| 0.60 - 0.79 | 強い相関 |
| 0.80 - 1.00 | 非常に強い相関 |

### 期待される知見

- **alcohol → quality**: 中程度の正の相関（r ≈ 0.48）。アルコール度数が高いワインほど品質評価が高い傾向
- **volatile_acidity → quality**: 弱〜中程度の負の相関（r ≈ -0.39）。揮発酸度が高いと酢のような風味が生じ評価が下がる
- **sulphates → quality**: 弱い正の相関（r ≈ 0.25）。適量の硫酸塩は保存性と風味に寄与
- **品質スコアの分布**: スコア 5 と 6 に約 82% が集中し、3 や 8 は稀少
- **外れ値**: residual_sugar（残糖 > 10 g/dm³）や chlorides（> 0.2 g/dm³）に明確な外れ値が存在
