# Example: California Housing（カリフォルニア住宅価格）データセット

## データセット

- **出典**: scikit-learn / StatLib / Pace, R. Kelley and Ronald Barry (1997) "Sparse Spatial Autoregressions", Statistics and Probability Letters
- **ライセンス**: Public Domain (StatLib)
- **件数**: 20,640 件の国勢調査ブロックグループ
- **特徴量**: 9 列（8 特徴量 + 1 目的変数）

### 使用する列

| 列名          | 説明                                      |
|---------------|-------------------------------------------|
| `MedInc`      | 世帯収入の中央値（万ドル単位）             |
| `HouseAge`    | 住宅の築年数の中央値                       |
| `AveRooms`    | 平均部屋数                                |
| `AveBedrms`   | 平均寝室数                                |
| `Population`  | ブロックグループの人口                     |
| `AveOccup`    | 平均居住人数                              |
| `Latitude`    | 緯度                                      |
| `Longitude`   | 経度                                      |
| `MedHouseVal` | 住宅価格の中央値（10万ドル単位、目的変数） |

## このサンプルで示せること

- **相関分析**: MedInc（所得）と MedHouseVal（住宅価格）の相関（r ≈ 0.69）。所得が住宅価格の最大の予測因子
- **分布分析**: MedHouseVal の天井効果。$500K（5.0）でキャップされたデータが集中する
- **外れ値検出**: AveOccup（平均居住人数）に極端な外れ値が存在する

### 期待される洞察

1. **MedInc → MedHouseVal**: 最も強い正の相関。高所得地域ほど住宅価格が高い
2. **HouseAge → MedHouseVal**: 弱い正の相関。古い住宅地域は都市中心部に近く、価格が高い傾向
3. **天井効果**: MedHouseVal = 5.0（$500K）にデータが集中。これはデータ収集時のキャップによるもの
4. **AveOccup 外れ値**: 大半のブロックは 2〜4 人だが、100 人以上の極端な値が存在（集団住宅や施設）
5. **Population 外れ値**: 一部のブロックグループに極端に多い人口が記録されている

## データの準備

```bash
cd examples/california_housing
# scikit-learn から直接取得するため download.sh は不要
python3 prepare.py
```

必要なパッケージ:

```bash
pip install scikit-learn pandas pyarrow
```

## 実行

```bash
# 所得と住宅価格の相関
sf run --recipe examples/california_housing/corr_income_price.toml

# 住宅価格の分布（天井効果の確認）
sf run --recipe examples/california_housing/price_distribution.toml

# 居住人数・人口の外れ値検出
sf run --recipe examples/california_housing/outlier_occupancy.toml

# CLI で直接実行する場合:
sf eda summary --source examples/california_housing/data/california_housing.parquet --cols MedInc,MedHouseVal,AveOccup
sf eda corr --source examples/california_housing/data/california_housing.parquet --cols MedInc,MedHouseVal --method pearson
```
