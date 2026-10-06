export type SourceType = "parquet" | "sqlite" | "postgres";
export type ScreenId =
  | "home"
  | "eda-summary"
  | "eda-correlation"
  | "eda-distribution"
  | "stats-outlier"
  | "run";
export type CorrMethod = "pearson" | "spearman";

export interface SourceConfig {
  type: SourceType;
  path: string;
  url: string;
  table: string;
  query: string;
  filter: string;
}

export interface ScreenConfig {
  id: ScreenId;
  path: string;
  navGroup: "Overview" | "EDA" | "Stats" | "Pipelines";
  label: string;
  eyebrow: string;
  title: string;
  description: string;
  purpose: string;
  accent: string;
}

export interface SummaryColumnStat {
  name: string;
  count: number;
  nullCount: number;
  mean: number | null;
  std: number | null;
  min: number | null;
  max: number | null;
  q25: number | null;
  q50: number | null;
  q75: number | null;
}

export interface SummaryResult {
  kind: "Summary";
  columns: SummaryColumnStat[];
}

export interface CorrelationResult {
  kind: "Correlation";
  method: CorrMethod;
  columns: string[];
  matrix: number[][];
}

export interface DistributionBin {
  lower: number;
  upper: number;
  count: number;
}

export interface DistributionResult {
  kind: "Distribution";
  column: string;
  bins: DistributionBin[];
}

export interface OutlierMetric {
  method: "iqr";
  totalRows: number;
  outlierCount: number;
  lowerBound: number;
  upperBound: number;
  q25: number;
  q75: number;
  iqr: number;
}

export interface OutlierResult {
  kind: "OutlierDetection";
  values: Record<string, OutlierMetric>;
}

export interface RunArtifactSummary {
  id: string;
  kind: string;
  summary: string;
}

export interface RunResult {
  execution: {
    tool: string;
    version: string;
    recipe: string;
    startedAt: string;
    stepsExecuted: number;
  };
  artifacts: RunArtifactSummary[];
}

export interface SummaryUiState {
  cols: string;
  error: string | null;
  lastRun: string;
  result: SummaryResult;
}

export interface CorrelationUiState {
  cols: string;
  method: CorrMethod;
  error: string | null;
  lastRun: string;
  result: CorrelationResult;
}

export interface DistributionUiState {
  col: string;
  bins: number;
  error: string | null;
  lastRun: string;
  result: DistributionResult;
}

export interface OutlierUiState {
  cols: string;
  error: string | null;
  lastRun: string;
  result: OutlierResult;
}

export interface RunUiState {
  recipePath: string;
  recipeToml: string;
  error: string | null;
  lastRun: string;
  result: RunResult;
}
