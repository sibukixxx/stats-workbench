# Example: FIFA 22 Players（FIFA選手）データセット

## データセット

- **出典**: [Kaggle — FIFA 22 Complete Player Dataset](https://www.kaggle.com/datasets/stefanoleone992/fifa-22-complete-player-dataset) / EA Sports FIFA 22
- **ライセンス**: Kaggle の利用規約に準拠
- **規模**: 約19,000選手、13個の特徴量
- **要件**: Kaggle CLI（`pip install kaggle`）または手動ダウンロード

### 使用カラム

| Column      | Description                    |
|-------------|--------------------------------|
| `overall`   | 総合評価 (0–99)               |
| `potential`  | ポテンシャル (0–99)           |
| `age`       | 年齢                           |
| `height_cm` | 身長 (cm)                      |
| `weight_kg` | 体重 (kg)                      |
| `wage_eur`  | 週給 (EUR)                     |
| `value_eur` | 市場価値 (EUR)                 |
| `pace`      | スピード (0–99)               |
| `shooting`  | シュート力 (0–99)             |
| `passing`   | パス精度 (0–99)               |
| `dribbling` | ドリブル力 (0–99)             |
| `defending` | 守備力 (0–99)                 |
| `physic`    | フィジカル (0–99)             |

## このサンプルが示すこと

- **相関分析**: 総合評価と各能力値の関係性（Pearson + Spearman）
- **分布分析**: 年齢のヒストグラムと要約統計
- **外れ値検出**: wage_eur, value_eur, overall の異常値検出

## データの準備

```bash
cd examples/fifa_players
./download.sh
python3 prepare.py
```

## 実行

```bash
# 能力値の相関分析
sf run --recipe examples/fifa_players/corr_ratings.toml

# 年齢の分布分析
sf run --recipe examples/fifa_players/dist_age.toml

# 外れ値検出（年俸、市場価値、総合評価）
sf run --recipe examples/fifa_players/outlier_wages.toml

# または直接実行:
sf eda corr --source examples/fifa_players/data/fifa_players.parquet \
    --cols overall,pace,shooting,passing,dribbling,defending,physic \
    --method pearson
```

## 結果の解釈

- **overall rating と各能力値の相関パターン**: shooting, passing, dribbling は overall との相関が高い傾向にある。一方 pace は相対的に相関が弱い（速さだけでは総合力は上がらない）
- **年齢分布は22〜25歳にピーク**: サッカー選手のキャリアライフサイクルを反映。若手の台頭と30代以降の引退による右裾の急減が見られる
- **wage_eur に極端な外れ値**: 一握りのスーパースター（メッシ、ロナウドなど）の年俸が桁違いに高い。スポーツ経済学における「勝者総取り」現象の典型例
- **value_eur も同様に右裾が長い**: 市場価値の分布は対数正規分布に近く、大多数の選手は低〜中程度の価値に集中する一方、トップ選手は数億ユーロに達する
- **defending と shooting の負の相関**: ポジションの専門性を反映。FW は shooting が高く defending が低い、DF はその逆のパターン
