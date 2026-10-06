# Example: Iris（アヤメ）データセット

## Dataset

- **Source**: [UCI Machine Learning Repository — Iris Data Set](https://archive.ics.uci.edu/ml/datasets/iris) / R.A. Fisher (1936)
- **License**: Public domain
- **Samples**: 150（3種 x 50サンプル）
- **Features**: 4つの数値特徴量

### 使用カラム

| Column         | Description              |
|----------------|--------------------------|
| `sepal_length` | 萼片の長さ (cm)          |
| `sepal_width`  | 萼片の幅 (cm)            |
| `petal_length` | 花弁の長さ (cm)          |
| `petal_width`  | 花弁の幅 (cm)            |

## この例で示すこと

- **花弁寸法の超強相関**: petal_length と petal_width の相関 r≈0.96 は「ほぼ完全な正の相関」
- **花弁長の二峰性分布**: setosa種（1-2cm）と versicolor/virginica種（3-7cm）の分離がデータから見える
- **萼片幅の外れ値検出**: IQR法による異常値フラグ付け

## Expected insights

- petal_length と petal_width の Pearson 相関 r≈0.96 は「ほぼ完全な正の相関」
- sepal_length と petal_length も r≈0.87 と強い相関
- petal_length の分布は明確な二峰性（1-2cmの setosa クラスタと 3-7cm の versicolor/virginica クラスタ）
- この二峰性は「データの中に2つの異なる母集団がある」ことを強く示唆
- sepal_width には IQR 基準で数点の外れ値が検出される可能性

## データの準備

```bash
cd examples/iris
./download.sh
python3 prepare.py
```

## 実行

```bash
# 基本統計量
sf run --recipe examples/iris/summary_all.toml

# 相関分析（Pearson + Spearman）
sf run --recipe examples/iris/corr_dimensions.toml

# 分布分析 + 外れ値検出
sf run --recipe examples/iris/dist_petal_length.toml

# Or run directly:
sf eda summary --source examples/iris/data/iris.parquet --cols sepal_length,sepal_width,petal_length,petal_width
sf eda corr --source examples/iris/data/iris.parquet --cols sepal_length,sepal_width,petal_length,petal_width --method pearson
```

## 結果の解釈

Iris データセットは統計学・機械学習の入門として最も有名なデータセットのひとつ。
Stats Workbench の EDA 機能だけでも以下の重要な構造が発見できる:

1. **花弁の寸法は萼片よりも品種を区別する力が強い** — 花弁間の相関が極めて高く、萼片間は低い
2. **petal_length の二峰性分布** は、ラベルなしでも「このデータには少なくとも2つの異なるグループがある」と示唆する
3. **外れ値検出** により、sepal_width で通常範囲から外れるサンプルを特定できる

これらの知見は、後続のクラスタリングや分類モデリングの前段階として有用。
