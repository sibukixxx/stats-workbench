# Example: Diamonds（ダイヤモンド）データセット

## データセット

- **出典**: [ggplot2 R パッケージ](https://ggplot2.tidyverse.org/reference/diamonds.html) / Wickham (2016)
- **ライセンス**: GPL (ggplot2 パッケージに付属)
- **件数**: 約 53,940 件のダイヤモンド
- **特徴量**: 7 つの数値列

### 使用する列

| 列名     | 説明                         |
|----------|------------------------------|
| `carat`  | カラット数（重量）            |
| `depth`  | 深さの割合 (%)               |
| `table`  | テーブル面の幅 (%)           |
| `price`  | 価格（USドル）               |
| `x`      | 長さ (mm)                    |
| `y`      | 幅 (mm)                     |
| `z`      | 深さ (mm)                    |

## このサンプルで示せること

- **相関分析**: carat と price の超強相関（r ≈ 0.92）。物理的な大きさ (x, y, z) も price と強い正の相関を持つ
- **分布分析**: price の極端な右裾分布（右に大きく歪んだ分布）。多くのダイヤモンドが低価格帯に集中し、高額なものは少数
- **外れ値検出**: 高級セグメント（高カラット・高価格）が IQR 法で外れ値として検出される

### 期待される洞察

1. **carat → price**: 最も強い相関。カラット数が価格の最大の決定要因であることを示す
2. **depth, table → price**: 相関は弱い。カット品質の指標だが、価格への直接的な影響は小さい
3. **価格分布**: 中央値と平均値の乖離が大きく、右裾が長い。対数変換が有効な典型的ケース
4. **外れ値**: 5 カラット以上や $15,000 以上の高額ダイヤモンドが外れ値として現れる

## データの準備

```bash
cd examples/diamonds
./download.sh
python3 prepare.py
```

## 実行

```bash
# 相関分析（Pearson + Spearman）
sf run --recipe examples/diamonds/corr_carat_price.toml

# 価格の分布
sf run --recipe examples/diamonds/dist_price.toml

# フルパイプライン
sf run --recipe examples/diamonds/pipeline_full.toml

# CLI で直接実行する場合:
sf eda summary --source examples/diamonds/data/diamonds.parquet --cols carat,price,depth,table
sf eda corr --source examples/diamonds/data/diamonds.parquet --cols carat,price,x,y,z --method pearson
```
