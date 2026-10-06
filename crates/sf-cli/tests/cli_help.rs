use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_shows_japanese_menu_descriptions() {
    Command::cargo_bin("sf")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("StatForge の分析メニューです"))
        .stdout(predicate::str::contains("探索的データ分析メニュー"))
        .stdout(predicate::str::contains("統計処理メニュー"))
        .stdout(predicate::str::contains("レシピ実行メニュー"));
}

#[test]
fn eda_help_shows_japanese_menu_descriptions() {
    Command::cargo_bin("sf")
        .unwrap()
        .args(["eda", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "データの全体像や傾向を素早く把握します",
        ))
        .stdout(predicate::str::contains("要約統計画面"))
        .stdout(predicate::str::contains("相関分析画面"))
        .stdout(predicate::str::contains("分布確認画面"));
}

#[test]
fn command_help_shows_japanese_purpose_text() {
    Command::cargo_bin("sf")
        .unwrap()
        .args(["eda", "summary", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "列ごとの件数、平均、最小値、最大値などを確認して、データの基本的な傾向を把握します",
        ));

    Command::cargo_bin("sf")
        .unwrap()
        .args(["stats", "outlier", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "IQR 法を使って外れ値候補を抽出し、異常な値や確認が必要なデータを見つけます",
        ));

    Command::cargo_bin("sf")
        .unwrap()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "TOML で定義した分析手順をまとめて実行し、再現可能なパイプラインとして利用します",
        ));
}
