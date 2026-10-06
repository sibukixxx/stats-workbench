# Example: Student Performance（学生成績）データセット

## データセット

- **出典**: [UCI Machine Learning Repository — Student Performance](https://archive.ics.uci.edu/ml/datasets/Student+Performance) / P. Cortez and A. Silva (2008)
- **ライセンス**: CC BY 4.0
- **件数**: 395 名の数学コース学生
- **特徴量**: 7 つの数値列

### 使用する列

| 列名        | 説明                              |
|-------------|-----------------------------------|
| `age`       | 学生の年齢（15〜22歳）            |
| `studytime` | 週あたり学習時間（1〜4のスケール） |
| `failures`  | 過去の不合格回数（0〜4）          |
| `absences`  | 欠席回数（0〜93）                 |
| `G1`        | 第1期成績（0〜20）               |
| `G2`        | 第2期成績（0〜20）               |
| `G3`        | 最終成績（0〜20）                |

## このサンプルで示せること

- **相関分析**: G1 → G2 → G3 の強い時系列的相関（r ≈ 0.80）。初期成績が最終成績を強く予測する
- **分布分析**: G3（最終成績）の二峰性分布。0 点付近と 10 点前後にピークが現れる
- **外れ値検出**: absences（欠席回数）に極端な外れ値が存在する

### 期待される洞察

1. **G1 → G3 相関**: 第1期の成績が最終成績の最良の予測因子。早期介入の重要性を示唆する
2. **studytime → G3**: 正の相関はあるが、G1 ほど強くない。学習時間だけでは成績は決まらない
3. **failures → G3**: 負の相関。過去の不合格経験は最終成績に悪影響を与える
4. **欠席外れ値**: 大半の学生は欠席が少ないが、一部に 40 回以上の極端な欠席者が存在する
5. **G3 二峰性**: 不合格者グループ（0 点付近）と通常グループの二極化が見られる

## データの準備

```bash
cd examples/student_performance
./download.sh
python3 prepare.py
```

## 実行

```bash
# 成績間の相関分析
stats-workbench run --recipe examples/student_performance/corr_grades.toml

# 最終成績の分布
stats-workbench run --recipe examples/student_performance/dist_final_grade.toml

# 欠席・不合格の外れ値検出
stats-workbench run --recipe examples/student_performance/outlier_absences.toml

# CLI で直接実行する場合:
stats-workbench eda summary --source examples/student_performance/data/student_performance.parquet --cols G1,G2,G3,absences
stats-workbench eda corr --source examples/student_performance/data/student_performance.parquet --cols G1,G2,G3 --method pearson
```
