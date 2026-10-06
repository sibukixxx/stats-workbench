# Example: Titanic 乗客データ

## Dataset

- **Source**: [Kaggle Titanic Competition](https://www.kaggle.com/c/titanic) (GitHub mirror)
- **License**: Public domain
- **Records**: 891 passengers (714 after dropping Age=null)

### Columns used

| Column        | Description                              |
|---------------|------------------------------------------|
| `Survived`    | 生存 (0=死亡, 1=生存)                    |
| `Pclass`      | 客室等級 (1=1等, 2=2等, 3=3等)           |
| `sex_encoded` | 性別 (0=male, 1=female)                  |
| `Age`         | 年齢                                     |
| `SibSp`       | 同乗の兄弟・配偶者数                     |
| `Parch`       | 同乗の親・子供数                         |
| `Fare`        | 運賃 (British pounds)                    |

## What this example demonstrates

- **基本統計量**: 乗客属性と運賃の記述統計
- **相関分析**: 運賃と生存の相関、客室等級と生存の相関
- **分布分析**: 運賃の右裾分布
- **外れ値検出**: IQR による高額運賃の外れ値検出

## Prepare data

```bash
cd examples/titanic
./download.sh
python3 prepare.py
```

## Run

```bash
# 基本統計量
sf run --recipe examples/titanic/summary_passengers.toml

# 生存率の相関分析
sf run --recipe examples/titanic/corr_survival.toml

# 運賃の分布と外れ値
sf run --recipe examples/titanic/fare_outliers.toml

# Or run directly:
sf eda summary --source examples/titanic/data/titanic.parquet --cols Survived,Pclass,sex_encoded,Age,Fare
sf eda corr --source examples/titanic/data/titanic.parquet --cols Survived,Pclass,sex_encoded,Fare --method pearson
```

## Expected insight

- **Pclass と Survived に中程度の負の相関** (r approximately -0.34): 上等クラスほど生存率が高い
- **Fare と Survived に正の相関**: 高運賃の乗客ほど生存
- **Fare の分布は強い右裾**: 大多数が 50 pounds 以下、一部 500 pounds 超の豪華客
- **IQR 外れ値検出**: 1等船室の高額運賃が外れ値として検出される

## 結果の解釈

### 相関係数の強さの目安

| |r| の範囲 | 解釈       |
|------------|------------|
| > 0.7      | 強い相関   |
| 0.4 - 0.7  | 中程度     |
| < 0.4      | 弱い相関   |

Pearson と Spearman の差が大きい場合は非線形関係を示唆する。
Pearson は線形関係のみを捉えるが、Spearman は単調関係（非線形含む）を捉える。
Titanic データでは Fare の分布が偏っているため、Spearman の方がより適切な場合がある。
