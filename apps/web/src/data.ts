import type {
  CorrMethod,
  CorrelationResult,
  DistributionResult,
  OutlierResult,
  RunResult,
  ScreenConfig,
  SourceConfig,
  SummaryColumnStat,
  SummaryResult,
} from "./types";

const DEFAULT_NUMERIC_COLUMNS = ["price", "quantity", "revenue"];

const STAT_LIBRARY: Record<string, SummaryColumnStat> = {
  price: {
    name: "price",
    count: 420,
    nullCount: 2,
    mean: 42.6,
    std: 11.4,
    min: 9.8,
    max: 88.2,
    q25: 34.1,
    q50: 42.0,
    q75: 50.4,
  },
  quantity: {
    name: "quantity",
    count: 420,
    nullCount: 0,
    mean: 3.8,
    std: 1.2,
    min: 1.0,
    max: 7.0,
    q25: 3.0,
    q50: 4.0,
    q75: 5.0,
  },
  revenue: {
    name: "revenue",
    count: 420,
    nullCount: 1,
    mean: 161.9,
    std: 78.5,
    min: 18.3,
    max: 462.1,
    q25: 102.2,
    q50: 148.6,
    q75: 210.4,
  },
  category: {
    name: "category",
    count: 420,
    nullCount: 0,
    mean: null,
    std: null,
    min: null,
    max: null,
    q25: null,
    q50: null,
    q75: null,
  },
};

export const SCREEN_CONFIG: ScreenConfig[] = [
  {
    id: "home",
    path: "/",
    navGroup: "Overview",
    label: "Overview",
    eyebrow: "StatForge",
    title: "分析メニュー全体を俯瞰するダッシュボード",
    description:
      "CLI のメニュー構成をブラウザ上で再整理し、探索的分析、統計処理、レシピ実行の入口をまとめて把握できます。",
    purpose:
      "いまどの分析へ進むべきかを判断するための起点画面です。各メニューの役割と入力条件をここで確認します。",
    accent: "Copper",
  },
  {
    id: "eda-summary",
    path: "/eda/summary",
    navGroup: "EDA",
    label: "Summary",
    eyebrow: "EDA",
    title: "要約統計画面",
    description:
      "列ごとの件数、平均、標準偏差、四分位数を確認して、データの基本傾向を短時間で掴むための画面です。",
    purpose:
      "どの列が分析対象として扱いやすいか、欠損やスケール感に問題がないかを最初に見極めます。",
    accent: "Teal",
  },
  {
    id: "eda-correlation",
    path: "/eda/correlation",
    navGroup: "EDA",
    label: "Correlation",
    eyebrow: "EDA",
    title: "相関分析画面",
    description:
      "複数列の相関係数を比較して、変数同士の関係の強さと方向を把握するための画面です。",
    purpose:
      "強い相関や冗長な指標を洗い出し、次の分析や特徴量設計の判断材料を得ます。",
    accent: "Moss",
  },
  {
    id: "eda-distribution",
    path: "/eda/distribution",
    navGroup: "EDA",
    label: "Distribution",
    eyebrow: "EDA",
    title: "分布確認画面",
    description:
      "指定した列のヒストグラムを表示し、ばらつき、偏り、分布の厚みを確認するための画面です。",
    purpose:
      "しきい値の決定、変換の要否、外れ値検知前の感触をここで掴みます。",
    accent: "Amber",
  },
  {
    id: "stats-outlier",
    path: "/stats/outlier",
    navGroup: "Stats",
    label: "Outlier",
    eyebrow: "Stats",
    title: "外れ値検出画面",
    description:
      "IQR 法で外れ値候補を抽出し、異常値や確認対象データを把握するための画面です。",
    purpose:
      "データ品質レビューや異常検知の前段として、どの列を精査すべきかを絞り込みます。",
    accent: "Brick",
  },
  {
    id: "run",
    path: "/run",
    navGroup: "Pipelines",
    label: "Recipe Run",
    eyebrow: "Pipelines",
    title: "レシピ実行画面",
    description:
      "TOML レシピを起点に複数ステップの分析をまとめて実行し、成果物を一覧で確認するための画面です。",
    purpose:
      "単発分析から再現可能なパイプラインに移す際の確認ポイントを一つの画面に集約します。",
    accent: "Indigo",
  },
];

export const SCREEN_BY_PATH = new Map(SCREEN_CONFIG.map((screen) => [screen.path, screen]));

export const DEFAULT_RECIPE_TOML = `[source]
type = "parquet"
path = "tests/fixtures/parquet/sales.parquet"

[[steps]]
id = "summary"
type = "eda.summary"
cols = ["price", "quantity", "revenue"]

[[steps]]
id = "corr"
type = "eda.correlation"
cols = ["price", "quantity", "revenue"]
method = "pearson"

[output]
terminal = true
format = "json"
path = "output/result.json"
artifacts = ["summary", "corr"]`;

export function createDefaultSourceConfig(): SourceConfig {
  return {
    type: "parquet",
    path: "tests/fixtures/parquet/sales.parquet",
    url: "postgresql://analyst@localhost:5432/statforge",
    table: "orders",
    query: "",
    filter: "price > 10",
  };
}

export function parseColumns(input: string, fallback = DEFAULT_NUMERIC_COLUMNS): string[] {
  const values = input
    .split(/[\n,]/)
    .map((value) => value.trim())
    .filter(Boolean);

  return values.length > 0 ? values : fallback;
}

export function buildSummaryResult(input: string): SummaryResult {
  const columns = parseColumns(input, ["price", "quantity", "revenue", "category"]).map(
    (name, index) => STAT_LIBRARY[name] ?? buildSyntheticColumnStat(name, index),
  );

  return { kind: "Summary", columns };
}

export function buildCorrelationResult(input: string, method: CorrMethod): CorrelationResult {
  const columns = parseColumns(input).slice(0, 5);
  const matrix = columns.map((left, rowIndex) =>
    columns.map((right, columnIndex) => {
      if (rowIndex === columnIndex) {
        return 1;
      }

      return correlationValue(left, right, method);
    }),
  );

  return { kind: "Correlation", method, columns, matrix };
}

export function buildDistributionResult(column: string, bins: number): DistributionResult {
  const safeBins = Math.max(1, bins);
  const seed = seedFrom(column || "revenue");
  const start = Math.round(seed / 2) + 10;
  const width = Math.max(6, Math.round((seed % 11) + 7));
  const counts = Array.from({ length: safeBins }, (_, index) =>
    Math.max(
      4,
      Math.round(
        8 +
          safeBins * 1.6 -
          Math.abs(index - (safeBins - 1) / 2) * 3 +
          ((seed + index * 17) % 7),
      ),
    ),
  );

  return {
    kind: "Distribution",
    column: column.trim() || "revenue",
    bins: counts.map((count, index) => ({
      lower: start + width * index,
      upper: start + width * (index + 1),
      count,
    })),
  };
}

export function buildOutlierResult(input: string): OutlierResult {
  const values = parseColumns(input, ["revenue"]).reduce<OutlierResult["values"]>((acc, name) => {
    const seed = seedFrom(name);
    const q25 = Math.round(seed * 2.1);
    const q75 = q25 + 18 + (seed % 14);
    const iqr = q75 - q25;

    acc[name] = {
      method: "iqr",
      totalRows: 420,
      outlierCount: (seed % 5) + 1,
      lowerBound: Number((q25 - 1.5 * iqr).toFixed(1)),
      upperBound: Number((q75 + 1.5 * iqr).toFixed(1)),
      q25,
      q75,
      iqr,
    };
    return acc;
  }, {});

  return { kind: "OutlierDetection", values };
}

export function buildRunResult(recipePath: string): RunResult {
  const fullRecipe = recipePath.includes("full");
  return {
    execution: {
      tool: "statforge-web-mock",
      version: "0.1.0",
      recipe: recipePath || "inline-recipe.toml",
      startedAt: "2026-03-22T20:45:00+09:00",
      stepsExecuted: fullRecipe ? 4 : 2,
    },
    artifacts: fullRecipe
      ? [
          { id: "summary", kind: "Stats", summary: "3 列の要約統計を生成" },
          { id: "corr", kind: "Stats", summary: "pearson 相関行列を生成" },
          { id: "dist", kind: "Stats", summary: "revenue の分布を 8 bins で生成" },
          { id: "outliers", kind: "Stats", summary: "revenue の外れ値候補を抽出" },
        ]
      : [
          { id: "summary", kind: "Stats", summary: "主要列の要約統計を生成" },
          { id: "corr", kind: "Stats", summary: "主要列の相関行列を生成" },
        ],
  };
}

function buildSyntheticColumnStat(name: string, index: number): SummaryColumnStat {
  const seed = seedFrom(name) + index * 3;
  const base = seed * 2.4;
  const isTextLike = /category|name|label|group/i.test(name);

  return {
    name,
    count: 420,
    nullCount: seed % 3,
    mean: isTextLike ? null : Number(base.toFixed(1)),
    std: isTextLike ? null : Number((base / 5.5).toFixed(1)),
    min: isTextLike ? null : Number((base * 0.4).toFixed(1)),
    max: isTextLike ? null : Number((base * 1.7).toFixed(1)),
    q25: isTextLike ? null : Number((base * 0.8).toFixed(1)),
    q50: isTextLike ? null : Number((base * 1.0).toFixed(1)),
    q75: isTextLike ? null : Number((base * 1.25).toFixed(1)),
  };
}

function correlationValue(left: string, right: string, method: CorrMethod): number {
  const leftSeed = seedFrom(left);
  const rightSeed = seedFrom(right);
  const combined = ((leftSeed * 13 + rightSeed * 7) % 160) / 100 - 0.8;
  const dampener = method === "pearson" ? 1 : 0.88;
  return Number(Math.max(-0.96, Math.min(0.96, combined * dampener)).toFixed(2));
}

function seedFrom(value: string): number {
  const normalized = value.trim().toLowerCase() || "seed";
  return normalized.split("").reduce((acc, char) => acc + char.charCodeAt(0), 0) % 37 + 9;
}
