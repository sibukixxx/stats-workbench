import { signal } from "@preact/signals";
import {
  DEFAULT_RECIPE_TOML,
  SCREEN_BY_PATH,
  buildCorrelationResult,
  buildDistributionResult,
  buildOutlierResult,
  buildRunResult,
  buildSummaryResult,
  createDefaultSourceConfig,
  parseColumns,
} from "./data";
import type {
  CorrelationUiState,
  DistributionUiState,
  OutlierUiState,
  RunUiState,
  SourceConfig,
  SummaryUiState,
} from "./types";

export const routeState = signal(normalizePath(currentPathname()));
export const sourceState = signal(createDefaultSourceConfig());
export const summaryState = signal<SummaryUiState>(createSummaryState());
export const correlationState = signal<CorrelationUiState>(createCorrelationState());
export const distributionState = signal<DistributionUiState>(createDistributionState());
export const outlierState = signal<OutlierUiState>(createOutlierState());
export const runState = signal<RunUiState>(createRunState());

export function attachRouteListener(): () => void {
  const handler = () => {
    routeState.value = normalizePath(currentPathname());
  };

  window.addEventListener("popstate", handler);
  return () => window.removeEventListener("popstate", handler);
}

export function navigate(path: string, replace = false): void {
  const next = normalizePath(path);
  const current = normalizePath(currentPathname());

  if (next === current) {
    routeState.value = next;
    return;
  }

  if (replace) {
    window.history.replaceState({}, "", next);
  } else {
    window.history.pushState({}, "", next);
  }
  routeState.value = next;
}

export function updateSource<K extends keyof SourceConfig>(key: K, value: SourceConfig[K]): void {
  sourceState.value = { ...sourceState.value, [key]: value };
}

export function updateSummary(cols: string): void {
  summaryState.value = { ...summaryState.value, cols };
}

export function updateCorrelationCols(cols: string): void {
  correlationState.value = { ...correlationState.value, cols };
}

export function updateCorrelationMethod(method: CorrelationUiState["method"]): void {
  correlationState.value = { ...correlationState.value, method };
}

export function updateDistributionCol(col: string): void {
  distributionState.value = { ...distributionState.value, col };
}

export function updateDistributionBins(bins: number): void {
  distributionState.value = { ...distributionState.value, bins };
}

export function updateOutlierCols(cols: string): void {
  outlierState.value = { ...outlierState.value, cols };
}

export function updateRecipePath(recipePath: string): void {
  runState.value = { ...runState.value, recipePath };
}

export function updateRecipeToml(recipeToml: string): void {
  runState.value = { ...runState.value, recipeToml };
}

export function executeSummary(): void {
  const sourceError = validateSource(sourceState.value);
  if (sourceError) {
    summaryState.value = { ...summaryState.value, error: sourceError };
    return;
  }

  summaryState.value = {
    ...summaryState.value,
    error: null,
    lastRun: runLabel("summary"),
    result: buildSummaryResult(summaryState.value.cols),
  };
}

export function executeCorrelation(): void {
  const sourceError = validateSource(sourceState.value);
  if (sourceError) {
    correlationState.value = { ...correlationState.value, error: sourceError };
    return;
  }

  const cols = parseColumns(correlationState.value.cols);
  if (cols.length < 2) {
    correlationState.value = {
      ...correlationState.value,
      error: "相関分析では 2 列以上を指定してください。",
    };
    return;
  }

  correlationState.value = {
    ...correlationState.value,
    error: null,
    lastRun: runLabel("correlation"),
    result: buildCorrelationResult(correlationState.value.cols, correlationState.value.method),
  };
}

export function executeDistribution(): void {
  const sourceError = validateSource(sourceState.value);
  if (sourceError) {
    distributionState.value = { ...distributionState.value, error: sourceError };
    return;
  }

  if (!distributionState.value.col.trim()) {
    distributionState.value = {
      ...distributionState.value,
      error: "分布確認では列名を指定してください。",
    };
    return;
  }

  if (distributionState.value.bins <= 0) {
    distributionState.value = {
      ...distributionState.value,
      error: "bins は 1 以上を指定してください。",
    };
    return;
  }

  distributionState.value = {
    ...distributionState.value,
    error: null,
    lastRun: runLabel("distribution"),
    result: buildDistributionResult(distributionState.value.col, distributionState.value.bins),
  };
}

export function executeOutlier(): void {
  const sourceError = validateSource(sourceState.value);
  if (sourceError) {
    outlierState.value = { ...outlierState.value, error: sourceError };
    return;
  }

  const cols = parseColumns(outlierState.value.cols, []);
  if (cols.length === 0) {
    outlierState.value = {
      ...outlierState.value,
      error: "外れ値検出では 1 列以上を指定してください。",
    };
    return;
  }

  outlierState.value = {
    ...outlierState.value,
    error: null,
    lastRun: runLabel("outlier"),
    result: buildOutlierResult(outlierState.value.cols),
  };
}

export function executeRun(): void {
  if (!runState.value.recipePath.trim() && !runState.value.recipeToml.trim()) {
    runState.value = {
      ...runState.value,
      error: "レシピパスまたは TOML 本文のいずれかを入力してください。",
    };
    return;
  }

  runState.value = {
    ...runState.value,
    error: null,
    lastRun: runLabel("recipe"),
    result: buildRunResult(runState.value.recipePath || "inline-recipe.toml"),
  };
}

export function resetAppState(path = "/"): void {
  sourceState.value = createDefaultSourceConfig();
  summaryState.value = createSummaryState();
  correlationState.value = createCorrelationState();
  distributionState.value = createDistributionState();
  outlierState.value = createOutlierState();
  runState.value = createRunState();
  navigate(path, true);
}

function createSummaryState(): SummaryUiState {
  const cols = "price, quantity, revenue, category";
  return {
    cols,
    error: null,
    lastRun: "初期プレビュー",
    result: buildSummaryResult(cols),
  };
}

function createCorrelationState(): CorrelationUiState {
  const cols = "price, quantity, revenue";
  return {
    cols,
    method: "pearson",
    error: null,
    lastRun: "初期プレビュー",
    result: buildCorrelationResult(cols, "pearson"),
  };
}

function createDistributionState(): DistributionUiState {
  return {
    col: "revenue",
    bins: 8,
    error: null,
    lastRun: "初期プレビュー",
    result: buildDistributionResult("revenue", 8),
  };
}

function createOutlierState(): OutlierUiState {
  const cols = "revenue";
  return {
    cols,
    error: null,
    lastRun: "初期プレビュー",
    result: buildOutlierResult(cols),
  };
}

function createRunState(): RunUiState {
  const recipePath = "tests/fixtures/recipes/eda_full.toml";
  return {
    recipePath,
    recipeToml: DEFAULT_RECIPE_TOML,
    error: null,
    lastRun: "初期プレビュー",
    result: buildRunResult(recipePath),
  };
}

function validateSource(source: SourceConfig): string | null {
  if (source.type === "parquet") {
    return source.path.trim() ? null : "Parquet ソースではファイルパスを入力してください。";
  }

  if (source.type === "sqlite") {
    if (!source.path.trim()) {
      return "SQLite ソースでは DB パスを入力してください。";
    }
    if (!source.table.trim() && !source.query.trim()) {
      return "SQLite ソースでは table か query のどちらかが必要です。";
    }
    return null;
  }

  if (!source.url.trim()) {
    return "PostgreSQL ソースでは接続 URL を入力してください。";
  }
  if (!source.table.trim() && !source.query.trim()) {
    return "PostgreSQL ソースでは table か query のどちらかが必要です。";
  }
  return null;
}

function currentPathname(): string {
  if (typeof window === "undefined") {
    return "/";
  }

  return window.location.pathname;
}

function normalizePath(path: string): string {
  const normalized =
    path.length > 1 && path.endsWith("/") ? path.slice(0, Math.max(path.length - 1, 1)) : path;

  return SCREEN_BY_PATH.has(normalized) ? normalized : "/";
}

function runLabel(kind: string): string {
  const time = new Date().toLocaleTimeString("ja-JP", {
    hour: "2-digit",
    minute: "2-digit",
  });

  return `${kind} mock executed at ${time}`;
}
