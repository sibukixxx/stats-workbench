# Example: Heart Disease（心臓病）データセット

## データセット

- **出典**: [UCI Machine Learning Repository — Heart Disease](https://archive.ics.uci.edu/ml/datasets/heart+disease)
- **提供元**: Cleveland Clinic Foundation（Robert Detrano, M.D., Ph.D.）
- **レコード数**: 約297（欠損値を含む行を除外後）
- **カラム数**: 14（13の特徴量 + 疾患有無の目的変数）

### カラム一覧

| カラム | 説明 |
|--------|------|
| `age` | 年齢（歳） |
| `sex` | 性別（1=男性, 0=女性） |
| `cp` | 胸痛タイプ（1-4） |
| `trestbps` | 安静時血圧（mmHg） |
| `chol` | 血清コレステロール（mg/dl） |
| `fbs` | 空腹時血糖 > 120 mg/dl（1=真, 0=偽） |
| `restecg` | 安静時心電図結果（0-2） |
| `thalach` | 最大心拍数 |
| `exang` | 運動誘発性狭心症（1=あり, 0=なし） |
| `oldpeak` | 運動によるST低下 |
| `slope` | 運動時ST部分の傾き |
| `ca` | 透視検査で着色された主要血管数（0-3） |
| `thal` | サラセミア（3=正常, 6=固定欠損, 7=可逆的欠損） |
| `target` | 心臓病の有無（1=あり, 0=なし） |

## このサンプルで確認できること

- **基本統計量**: 年齢、血圧、コレステロール、最大心拍数のばらつきと中心傾向
- **相関分析**: 年齢・血圧・コレステロール・最大心拍数と心臓病の関係
- **外れ値検出**: コレステロールや血圧の臨床的に異常な値の特定

## データ準備

```bash
cd examples/heart_disease
./download.sh
python3 prepare.py
```

## 実行

```bash
# 基本統計量: リスク因子の概要
stats-workbench run --recipe examples/heart_disease/summary_risk_factors.toml

# 相関分析: 各因子と心臓病の関係
stats-workbench run --recipe examples/heart_disease/corr_heart.toml

# 外れ値検出: バイタルサインの異常値
stats-workbench run --recipe examples/heart_disease/outlier_vitals.toml

# CLI で直接実行する場合:
stats-workbench eda summary --source examples/heart_disease/data/heart_disease.parquet --cols age,trestbps,chol,thalach
stats-workbench eda corr --source examples/heart_disease/data/heart_disease.parquet --cols age,chol,thalach,target --method pearson
stats-workbench stats outlier --source examples/heart_disease/data/heart_disease.parquet --cols chol,trestbps
```

## 結果の解釈

### 主要な相関関係

| 因子の組み合わせ | 期待される相関 | 臨床的意味 |
|-----------------|---------------|-----------|
| thalach → target | 負の相関（r ≈ -0.42） | 最大心拍数が低い患者ほど心臓病の可能性が高い |
| oldpeak → target | 正の相関（r ≈ 0.43） | ST低下が大きいほど心臓病リスクが高い |
| age → target | 弱い正の相関（r ≈ 0.23） | 加齢に伴いリスクが上昇 |
| chol → target | ほぼ無相関（r ≈ 0.09） | コレステロール単独では予測力が弱い |

### 外れ値の臨床的意味

- **コレステロール > 400 mg/dl**: 家族性高コレステロール血症の可能性。治療介入が必要な水準
- **安静時血圧 > 170 mmHg**: 高血圧ステージ3。心血管イベントの高リスク群
- **最大心拍数 < 100**: 年齢に対して著しく低い場合、心機能低下を示唆

### 注意事項

- このデータセットは1988年のもので、現在の診断基準とは異なる場合がある
- 欠損値を含む行を除外しているため、サンプルサイズに偏りが生じている可能性がある
- 二値分類（target: 0/1）に変換済み（元データは 0-4 の重症度スケール）
