# Example: Spotify Tracks（Spotify楽曲）データセット

## データセット

- **出典**: [Kaggle — Spotify Tracks Dataset](https://www.kaggle.com/datasets/maharshipandya/spotify-tracks-dataset) / Spotify Web API
- **ライセンス**: Kaggle の利用規約に準拠
- **規模**: 約114,000曲、11個のオーディオ特徴量
- **要件**: Kaggle CLI（`pip install kaggle`）または手動ダウンロード

### 使用カラム

| Column            | Description                              |
|-------------------|------------------------------------------|
| `danceability`    | ダンスのしやすさ (0.0–1.0)               |
| `energy`          | エネルギー感 (0.0–1.0)                   |
| `loudness`        | 音圧レベル (dB、通常 -60〜0)             |
| `speechiness`     | 音声（スピーチ）の割合 (0.0–1.0)         |
| `acousticness`    | アコースティック度 (0.0–1.0)             |
| `instrumentalness`| インストゥルメンタル度 (0.0–1.0)         |
| `liveness`        | ライブ感 (0.0–1.0)                       |
| `valence`         | ポジティブ度 (0.0–1.0)                   |
| `tempo`           | テンポ (BPM)                             |
| `duration_ms`     | 曲の長さ (ミリ秒)                        |
| `popularity`      | 人気度 (0–100)                           |

## このサンプルが示すこと

- **相関分析**: オーディオ特徴量間の関係性（Pearson + Spearman）
- **分布分析**: popularity のヒストグラムと要約統計
- **外れ値検出**: duration_ms, tempo, loudness の異常値検出

## データの準備

```bash
cd examples/spotify_tracks
./download.sh
python3 prepare.py
```

## 実行

```bash
# オーディオ特徴量の相関分析
stats-workbench run --recipe examples/spotify_tracks/corr_audio_features.toml

# popularity の分布分析
stats-workbench run --recipe examples/spotify_tracks/dist_popularity.toml

# 外れ値検出（duration_ms, tempo, loudness）
stats-workbench run --recipe examples/spotify_tracks/outlier_duration.toml

# または直接実行:
stats-workbench eda corr --source examples/spotify_tracks/data/spotify_tracks.parquet \
    --cols energy,loudness,danceability,acousticness,valence,speechiness,popularity \
    --method pearson
```

## 結果の解釈

- **energy と loudness に強い正の相関** (r≈0.76): エネルギッシュな曲ほど音圧が高い。これは音楽制作の物理的性質を反映している
- **acousticness と energy に強い負の相関**: アコースティック曲はエネルギー値が低い傾向。エレクトロニック/ロック系とアコースティック系の二極化が見える
- **popularity はべき分布**: 大多数の曲は低い人気度に集中し、一部のヒット曲だけが高い人気度を持つ。音楽市場の「勝者総取り」構造を反映
- **duration_ms の外れ値**: 異常に長い曲（30分以上など）はポッドキャストやアンビエント音楽が混入している可能性がある。データクリーニングの候補として要確認
- **tempo は比較的正規分布に近い**: 多くの曲が100–140BPM付近に集中するが、極端に遅い/速い曲も存在する
