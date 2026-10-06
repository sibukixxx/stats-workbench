import type { ComponentChildren } from "preact";
import { useEffect } from "preact/hooks";
import { SCREEN_CONFIG, SCREEN_BY_PATH } from "./data";
import {
  attachRouteListener,
  correlationState,
  distributionState,
  executeCorrelation,
  executeDistribution,
  executeOutlier,
  executeRun,
  executeSummary,
  navigate,
  outlierState,
  routeState,
  runState,
  sourceState,
  summaryState,
  updateCorrelationCols,
  updateCorrelationMethod,
  updateDistributionBins,
  updateDistributionCol,
  updateOutlierCols,
  updateRecipePath,
  updateRecipeToml,
  updateSource,
  updateSummary,
} from "./state";
import type {
  CorrelationResult,
  DistributionResult,
  OutlierResult,
  RunResult,
  ScreenConfig,
  SourceConfig,
  SummaryResult,
} from "./types";

const NAV_GROUPS = ["Overview", "EDA", "Stats", "Pipelines"] as const;

export function App() {
  useEffect(() => attachRouteListener(), []);

  const activeScreen = SCREEN_BY_PATH.get(routeState.value) ?? SCREEN_CONFIG[0];

  return (
    <div className="app-shell text-[var(--color-ink)]">
      <div className="mx-auto flex min-h-screen max-w-[1600px] flex-col gap-4 px-4 py-4 lg:flex-row lg:px-6 lg:py-6">
        <Sidebar activePath={activeScreen.path} />
        <main className="flex-1">
          <div className="space-y-4">
            <PageHero screen={activeScreen} />
            <ScreenContent screen={activeScreen} />
          </div>
        </main>
      </div>
    </div>
  );
}

function Sidebar(props: { activePath: string }) {
  return (
    <aside className="panel sidebar-panel w-full overflow-hidden lg:sticky lg:top-6 lg:h-[calc(100vh-3rem)] lg:w-[320px]">
      <div className="space-y-6 p-5 lg:p-6">
        <div className="space-y-3">
          <p className="eyebrow text-[var(--color-highlight)]">Stats Workbench Web</p>
          <div>
            <h1 className="font-serif text-3xl leading-tight text-white">
              Analysis
              <br />
              Atelier
            </h1>
            <p className="mt-3 text-sm leading-6 text-white/72">
              CLI の操作意図を崩さずに、分析の入口、条件設定、結果確認を一つの流れにまとめた
              UI モックです。
            </p>
          </div>
        </div>

        {NAV_GROUPS.map((group) => {
          const screens = SCREEN_CONFIG.filter((screen) => screen.navGroup === group);
          if (screens.length === 0) {
            return null;
          }

          return (
            <div className="space-y-3" key={group}>
              <p className="text-xs font-semibold uppercase tracking-[0.3em] text-white/45">
                {group}
              </p>
              <div className="space-y-2">
                {screens.map((screen) => {
                  const active = screen.path === props.activePath;
                  return (
                    <button
                      type="button"
                      key={screen.path}
                      className={`w-full rounded-[24px] border px-4 py-3 text-left transition ${
                        active
                          ? "border-white/55 bg-white/16 text-white shadow-[0_18px_48px_rgba(15,23,42,0.2)]"
                          : "border-white/10 bg-white/4 text-white/70 hover:border-white/30 hover:bg-white/10"
                      }`}
                      aria-current={active ? "page" : undefined}
                      onClick={() => navigate(screen.path)}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <div className="text-[0.7rem] uppercase tracking-[0.26em] text-white/45">
                            {screen.eyebrow}
                          </div>
                          <div className="mt-1 text-base font-semibold">{screen.label}</div>
                        </div>
                        <span className="chip bg-white/10 text-white/80">{screen.accent}</span>
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>
          );
        })}

        <div className="rounded-[24px] border border-white/10 bg-white/6 p-4 text-sm text-white/72">
          <div className="eyebrow mb-2 text-white/40">Active Source</div>
          <div className="font-medium text-white">{sourceTitle(sourceState.value)}</div>
          <div className="mt-2 leading-6">{sourceLocation(sourceState.value)}</div>
        </div>
      </div>
    </aside>
  );
}

function PageHero(props: { screen: ScreenConfig }) {
  const source = sourceState.value;

  return (
    <section className="panel relative overflow-hidden p-6 lg:p-8">
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.3fr)_minmax(320px,0.7fr)]">
        <div className="space-y-4">
          <div className="flex flex-wrap items-center gap-3">
            <span className="chip bg-[var(--color-wash)] text-[var(--color-ink)]">
              {props.screen.eyebrow}
            </span>
            <span className="chip bg-white/65 text-[var(--color-soft-ink)]">
              {props.screen.accent}
            </span>
          </div>
          <div className="space-y-3">
            <p className="eyebrow text-[var(--color-soft-ink)]">{props.screen.label}</p>
            <h2 className="max-w-4xl font-serif text-3xl leading-tight text-[var(--color-ink)] lg:text-[2.6rem]">
              {props.screen.title}
            </h2>
            <p className="max-w-3xl text-[1.02rem] leading-7 text-[var(--color-soft-ink)]">
              {props.screen.description}
            </p>
          </div>
        </div>

        <div className="rounded-[28px] border border-[var(--color-border)] bg-white/72 p-5 shadow-[0_24px_70px_rgba(15,23,42,0.08)]">
          <p className="eyebrow text-[var(--color-soft-ink)]">Screen Purpose</p>
          <p className="mt-3 text-sm leading-7 text-[var(--color-soft-ink)]">{props.screen.purpose}</p>
          <dl className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-1">
            <div className="rounded-[20px] bg-[var(--color-wash)]/70 p-4">
              <dt className="eyebrow text-[var(--color-soft-ink)]">Source Type</dt>
              <dd className="mt-1 text-base font-semibold text-[var(--color-ink)]">
                {source.type}
              </dd>
            </div>
            <div className="rounded-[20px] bg-[var(--color-wash)]/70 p-4">
              <dt className="eyebrow text-[var(--color-soft-ink)]">Target</dt>
              <dd className="mt-1 text-base font-semibold text-[var(--color-ink)]">
                {source.type === "postgres" ? source.url : source.path}
              </dd>
            </div>
          </dl>
        </div>
      </div>
    </section>
  );
}

function ScreenContent(props: { screen: ScreenConfig }) {
  switch (props.screen.id) {
    case "home":
      return <HomePage />;
    case "eda-summary":
      return <SummaryPage />;
    case "eda-correlation":
      return <CorrelationPage />;
    case "eda-distribution":
      return <DistributionPage />;
    case "stats-outlier":
      return <OutlierPage />;
    case "run":
      return <RunPage />;
    default:
      return null;
  }
}

function HomePage() {
  const primaryScreens = SCREEN_CONFIG.filter((screen) => screen.id !== "home");

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(340px,0.85fr)]">
      <section className="panel p-6 lg:p-8">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="eyebrow text-[var(--color-soft-ink)]">Navigation</p>
            <h3 className="mt-2 font-serif text-2xl text-[var(--color-ink)]">
              どの分析へ進むかをここで決める
            </h3>
          </div>
          <span className="chip bg-[var(--color-wash)] text-[var(--color-soft-ink)]">
            5 screens
          </span>
        </div>

        <div className="mt-6 grid gap-4 md:grid-cols-2">
          {primaryScreens.map((screen) => (
            <button
              type="button"
              key={screen.id}
              className="group rounded-[28px] border border-[var(--color-border)] bg-white/72 p-5 text-left transition hover:-translate-y-0.5 hover:border-[var(--color-ink)]/20 hover:shadow-[0_22px_60px_rgba(15,23,42,0.1)]"
              onClick={() => navigate(screen.path)}
            >
              <div className="flex items-center justify-between gap-3">
                <span className="chip bg-[var(--color-wash)] text-[var(--color-soft-ink)]">
                  {screen.navGroup}
                </span>
                <span className="text-sm text-[var(--color-soft-ink)] transition group-hover:text-[var(--color-ink)]">
                  Open
                </span>
              </div>
              <h4 className="mt-4 font-serif text-2xl text-[var(--color-ink)]">{screen.title}</h4>
              <p className="mt-2 text-xs font-semibold uppercase tracking-[0.24em] text-[var(--color-soft-ink)]">
                {screen.label}
              </p>
              <p className="mt-3 text-sm leading-7 text-[var(--color-soft-ink)]">
                {screen.description}
              </p>
            </button>
          ))}
        </div>
      </section>

      <div className="space-y-4">
        <SourceSnapshot />
        <QuickPatternCard />
      </div>
    </div>
  );
}

function SummaryPage() {
  const state = summaryState.value;
  return (
    <TwoColumnLayout
      left={
        <>
          <SourceFormCard />
          <ConfigCard
            title="Summary Controls"
            description="カンマ区切りまたは改行区切りで対象列を入力します。空の場合は既定列のプレビューを表示します。"
            actionLabel="要約統計をモック実行"
            actionMeta={state.lastRun}
            error={state.error}
            onAction={executeSummary}
          >
            <TextAreaField
              label="Columns"
              value={state.cols}
              onInput={(value) => updateSummary(value)}
              placeholder="price, quantity, revenue"
            />
          </ConfigCard>
        </>
      }
      right={<SummaryResultPanel result={state.result} />}
    />
  );
}

function CorrelationPage() {
  const state = correlationState.value;
  return (
    <TwoColumnLayout
      left={
        <>
          <SourceFormCard />
          <ConfigCard
            title="Correlation Controls"
            description="2 列以上を指定すると、Pearson / Spearman の行列を即座にプレビューできます。"
            actionLabel="相関分析をモック実行"
            actionMeta={state.lastRun}
            error={state.error}
            onAction={executeCorrelation}
          >
            <TextAreaField
              label="Columns"
              value={state.cols}
              onInput={(value) => updateCorrelationCols(value)}
              placeholder="price, quantity, revenue"
            />
            <SelectField
              label="Method"
              value={state.method}
              options={[
                { label: "Pearson", value: "pearson" },
                { label: "Spearman", value: "spearman" },
              ]}
              onChange={(value) => updateCorrelationMethod(value as "pearson" | "spearman")}
            />
          </ConfigCard>
        </>
      }
      right={<CorrelationResultPanel result={state.result} />}
    />
  );
}

function DistributionPage() {
  const state = distributionState.value;
  return (
    <TwoColumnLayout
      left={
        <>
          <SourceFormCard />
          <ConfigCard
            title="Distribution Controls"
            description="列名と bins 数を指定すると、棒グラフ風のカードで分布の厚みを確認できます。"
            actionLabel="分布確認をモック実行"
            actionMeta={state.lastRun}
            error={state.error}
            onAction={executeDistribution}
          >
            <TextField
              label="Column"
              value={state.col}
              onInput={(value) => updateDistributionCol(value)}
              placeholder="revenue"
            />
            <NumberField
              label="Bins"
              value={state.bins}
              min={1}
              onInput={(value) => updateDistributionBins(value)}
            />
          </ConfigCard>
        </>
      }
      right={<DistributionResultPanel result={state.result} />}
    />
  );
}

function OutlierPage() {
  const state = outlierState.value;
  return (
    <TwoColumnLayout
      left={
        <>
          <SourceFormCard />
          <ConfigCard
            title="Outlier Controls"
            description="複数列を指定すると、列ごとの IQR 境界と外れ値件数を並べて比較できます。"
            actionLabel="外れ値検出をモック実行"
            actionMeta={state.lastRun}
            error={state.error}
            onAction={executeOutlier}
          >
            <TextAreaField
              label="Columns"
              value={state.cols}
              onInput={(value) => updateOutlierCols(value)}
              placeholder="revenue"
            />
          </ConfigCard>
        </>
      }
      right={<OutlierResultPanel result={state.result} />}
    />
  );
}

function RunPage() {
  const state = runState.value;
  return (
    <TwoColumnLayout
      left={
        <>
          <ConfigCard
            title="Recipe Runner"
            description="レシピパスと TOML 本文を調整し、実行メタ情報と artifact 一覧をブラウザ上で検証します。"
            actionLabel="レシピをモック実行"
            actionMeta={state.lastRun}
            error={state.error}
            onAction={executeRun}
          >
            <TextField
              label="Recipe Path"
              value={state.recipePath}
              onInput={(value) => updateRecipePath(value)}
              placeholder="tests/fixtures/recipes/eda_full.toml"
            />
            <TextAreaField
              label="Recipe TOML"
              value={state.recipeToml}
              onInput={(value) => updateRecipeToml(value)}
              placeholder='[source]\ntype = "parquet"'
              rows={14}
            />
          </ConfigCard>
        </>
      }
      right={<RunResultPanel result={state.result} />}
    />
  );
}

function TwoColumnLayout(props: { left: ComponentChildren; right: ComponentChildren }) {
  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
      {props.left}
      {props.right}
    </div>
  );
}

function SourceSnapshot() {
  const source = sourceState.value;
  return (
    <section className="panel p-6">
      <p className="eyebrow text-[var(--color-soft-ink)]">Source Snapshot</p>
      <h3 className="mt-2 font-serif text-2xl text-[var(--color-ink)]">現在の入力前提</h3>
      <div className="mt-5 space-y-4">
        <dl className="grid gap-3 sm:grid-cols-2">
          <MetricCell label="Type" value={source.type} />
          <MetricCell label="Path / URL" value={sourceLocation(source)} />
          <MetricCell label="Table" value={source.table || "not set"} />
          <MetricCell label="Query" value={source.query || "not set"} />
        </dl>
      </div>
    </section>
  );
}

function QuickPatternCard() {
  return (
    <section className="panel p-6">
      <p className="eyebrow text-[var(--color-soft-ink)]">Workflow Hint</p>
      <h3 className="mt-2 font-serif text-2xl text-[var(--color-ink)]">推奨フロー</h3>
      <div className="mt-5 grid gap-3">
        {[
          "1. Summary で欠損とスケール感を確認",
          "2. Correlation で関係性の強い列を洗い出す",
          "3. Distribution / Outlier で閾値と異常値を確認",
          "4. Recipe Run で再現可能な手順に固める",
        ].map((step) => (
          <div
            key={step}
            className="rounded-[22px] border border-[var(--color-border)] bg-white/66 px-4 py-3 text-sm leading-7 text-[var(--color-soft-ink)]"
          >
            {step}
          </div>
        ))}
      </div>
    </section>
  );
}

function SourceFormCard() {
  const source = sourceState.value;

  return (
    <section className="panel p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="eyebrow text-[var(--color-soft-ink)]">Shared Source Form</p>
          <h3 className="mt-2 font-serif text-2xl text-[var(--color-ink)]">データソース設定</h3>
        </div>
        <span className="chip bg-[var(--color-wash)] text-[var(--color-soft-ink)]">
          Common
        </span>
      </div>

      <div className="mt-5 flex flex-wrap gap-2">
        {([
          ["parquet", "Parquet"],
          ["sqlite", "SQLite"],
          ["postgres", "PostgreSQL"],
        ] as const).map(([value, label]) => {
          const active = source.type === value;
          return (
            <button
              type="button"
              key={value}
              className={`rounded-full px-4 py-2 text-sm font-semibold transition ${
                active
                  ? "bg-[var(--color-ink)] text-white"
                  : "bg-[var(--color-wash)] text-[var(--color-soft-ink)] hover:bg-[var(--color-border)]"
              }`}
              onClick={() => updateSource("type", value)}
            >
              {label}
            </button>
          );
        })}
      </div>

      <div className="mt-5 grid gap-4">
        {source.type === "parquet" ? (
          <TextField
            label="Parquet Path"
            value={source.path}
            onInput={(value) => updateSource("path", value)}
            placeholder="tests/fixtures/parquet/sales.parquet"
          />
        ) : null}

        {source.type === "sqlite" ? (
          <>
            <TextField
              label="SQLite Path"
              value={source.path}
              onInput={(value) => updateSource("path", value)}
              placeholder="sqlite://data/app.db"
            />
            <TextField
              label="Table"
              value={source.table}
              onInput={(value) => updateSource("table", value)}
              placeholder="orders"
            />
            <TextAreaField
              label="Query"
              value={source.query}
              onInput={(value) => updateSource("query", value)}
              placeholder="SELECT * FROM orders LIMIT 100"
              rows={4}
            />
          </>
        ) : null}

        {source.type === "postgres" ? (
          <>
            <TextField
              label="PostgreSQL URL"
              value={source.url}
              onInput={(value) => updateSource("url", value)}
              placeholder="postgresql://user:pass@localhost:5432/app"
            />
            <TextField
              label="Table"
              value={source.table}
              onInput={(value) => updateSource("table", value)}
              placeholder="orders"
            />
            <TextAreaField
              label="Query"
              value={source.query}
              onInput={(value) => updateSource("query", value)}
              placeholder="SELECT price, quantity, revenue FROM orders"
              rows={4}
            />
            <TextField
              label="Filter"
              value={source.filter}
              onInput={(value) => updateSource("filter", value)}
              placeholder="price > 10"
            />
          </>
        ) : null}
      </div>
    </section>
  );
}

function ConfigCard(props: {
  title: string;
  description: string;
  actionLabel: string;
  actionMeta: string;
  error: string | null;
  onAction: () => void;
  children: ComponentChildren;
}) {
  return (
    <section className="panel p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="eyebrow text-[var(--color-soft-ink)]">Configuration</p>
          <h3 className="mt-2 font-serif text-2xl text-[var(--color-ink)]">{props.title}</h3>
        </div>
        <span className="chip bg-[var(--color-wash)] text-[var(--color-soft-ink)]">
          {props.actionMeta}
        </span>
      </div>
      <p className="mt-3 text-sm leading-7 text-[var(--color-soft-ink)]">{props.description}</p>
      <div className="mt-5 grid gap-4">{props.children}</div>
      {props.error ? (
        <div className="mt-5 rounded-[20px] border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
          {props.error}
        </div>
      ) : null}
      <div className="mt-5 flex flex-wrap items-center justify-between gap-3">
        <p className="text-sm leading-7 text-[var(--color-soft-ink)]">
          実行はモックですが、バリデーション条件は CLI の入力ルールに合わせています。
        </p>
        <button type="button" className="action-button" onClick={props.onAction}>
          {props.actionLabel}
        </button>
      </div>
    </section>
  );
}

function SummaryResultPanel(props: { result: SummaryResult }) {
  return (
    <section className="panel p-6 lg:p-7">
      <PanelHeader
        eyebrow="Result"
        title="Summary Preview"
        detail={`${props.result.columns.length} columns`}
      />
      <div className="mt-5 overflow-hidden rounded-[24px] border border-[var(--color-border)]">
        <div className="overflow-x-auto">
          <table className="min-w-full text-left text-sm">
            <thead className="bg-[var(--color-wash)] text-[var(--color-soft-ink)]">
              <tr>
                {["Column", "Count", "Null", "Mean", "Std", "Min", "Median", "Max"].map(
                  (heading) => (
                    <th className="px-4 py-3 font-semibold" key={heading}>
                      {heading}
                    </th>
                  ),
                )}
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-border)] bg-white/72">
              {props.result.columns.map((column) => (
                <tr key={column.name}>
                  <td className="px-4 py-3 font-semibold text-[var(--color-ink)]">{column.name}</td>
                  <td className="px-4 py-3">{column.count}</td>
                  <td className="px-4 py-3">{column.nullCount}</td>
                  <td className="px-4 py-3">{formatMetric(column.mean)}</td>
                  <td className="px-4 py-3">{formatMetric(column.std)}</td>
                  <td className="px-4 py-3">{formatMetric(column.min)}</td>
                  <td className="px-4 py-3">{formatMetric(column.q50)}</td>
                  <td className="px-4 py-3">{formatMetric(column.max)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}

function CorrelationResultPanel(props: { result: CorrelationResult }) {
  return (
    <section className="panel p-6 lg:p-7">
      <PanelHeader
        eyebrow="Result"
        title="Correlation Matrix"
        detail={`${props.result.method} / ${props.result.columns.length} columns`}
      />
      <div className="mt-5 overflow-hidden rounded-[24px] border border-[var(--color-border)] bg-white/72 p-3">
        <div className="grid gap-2" style={{ gridTemplateColumns: `140px repeat(${props.result.columns.length}, minmax(90px, 1fr))` }}>
          <div />
          {props.result.columns.map((column) => (
            <div key={`heading-${column}`} className="px-2 py-3 text-center text-sm font-semibold text-[var(--color-soft-ink)]">
              {column}
            </div>
          ))}
          {props.result.columns.map((rowName, rowIndex) => (
            <div key={`row-block-${rowName}`} className="contents">
              <div
                className="rounded-[18px] bg-[var(--color-wash)] px-3 py-3 text-sm font-semibold text-[var(--color-ink)]"
              >
                {rowName}
              </div>
              {props.result.matrix[rowIndex].map((value, columnIndex) => (
                <div
                  key={`${rowName}-${props.result.columns[columnIndex]}`}
                  className="rounded-[18px] px-3 py-4 text-center text-sm font-semibold text-white shadow-[inset_0_0_0_1px_rgba(255,255,255,0.14)]"
                  style={{ backgroundColor: correlationColor(value) }}
                >
                  {value.toFixed(2)}
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function DistributionResultPanel(props: { result: DistributionResult }) {
  const max = Math.max(...props.result.bins.map((bin) => bin.count));

  return (
    <section className="panel p-6 lg:p-7">
      <PanelHeader
        eyebrow="Result"
        title="Distribution Preview"
        detail={`${props.result.column} / ${props.result.bins.length} bins`}
      />
      <div className="mt-5 space-y-3">
        {props.result.bins.map((bin, index) => (
          <div
            key={`${bin.lower}-${bin.upper}`}
            className="rounded-[24px] border border-[var(--color-border)] bg-white/74 px-4 py-4"
          >
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-sm font-semibold text-[var(--color-ink)]">
                Bin {index + 1}: {bin.lower.toFixed(0)} - {bin.upper.toFixed(0)}
              </div>
              <div className="text-sm text-[var(--color-soft-ink)]">{bin.count} rows</div>
            </div>
            <div className="mt-3 h-3 overflow-hidden rounded-full bg-[var(--color-wash)]">
              <div
                className="h-full rounded-full bg-[linear-gradient(90deg,var(--color-highlight),var(--color-signal))]"
                style={{ width: `${(bin.count / max) * 100}%` }}
              />
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function OutlierResultPanel(props: { result: OutlierResult }) {
  const rows = Object.entries(props.result.values);

  return (
    <section className="panel p-6 lg:p-7">
      <PanelHeader eyebrow="Result" title="Outlier Preview" detail={`${rows.length} columns`} />
      <div className="mt-5 grid gap-4 md:grid-cols-2">
        {rows.map(([name, value]) => (
          <article
            key={name}
            className="rounded-[26px] border border-[var(--color-border)] bg-white/76 p-5"
          >
            <div className="flex items-center justify-between gap-3">
              <h4 className="font-serif text-2xl text-[var(--color-ink)]">{name}</h4>
              <span className="chip bg-[var(--color-wash)] text-[var(--color-soft-ink)]">
                {value.method}
              </span>
            </div>
            <div className="mt-5 grid gap-3 sm:grid-cols-2">
              <MetricCell label="Rows" value={String(value.totalRows)} />
              <MetricCell label="Outliers" value={String(value.outlierCount)} />
              <MetricCell label="Lower" value={value.lowerBound.toFixed(1)} />
              <MetricCell label="Upper" value={value.upperBound.toFixed(1)} />
              <MetricCell label="Q25" value={value.q25.toFixed(1)} />
              <MetricCell label="Q75" value={value.q75.toFixed(1)} />
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function RunResultPanel(props: { result: RunResult }) {
  return (
    <section className="panel p-6 lg:p-7">
      <PanelHeader
        eyebrow="Result"
        title="Recipe Execution Preview"
        detail={`${props.result.execution.stepsExecuted} steps`}
      />
      <div className="mt-5 grid gap-4 md:grid-cols-2">
        <section className="rounded-[26px] border border-[var(--color-border)] bg-white/76 p-5">
          <p className="eyebrow text-[var(--color-soft-ink)]">Execution Meta</p>
          <dl className="mt-4 grid gap-3">
            <MetricCell label="Tool" value={props.result.execution.tool} />
            <MetricCell label="Version" value={props.result.execution.version} />
            <MetricCell label="Recipe" value={props.result.execution.recipe} />
            <MetricCell label="Started" value={props.result.execution.startedAt} />
          </dl>
        </section>
        <section className="rounded-[26px] border border-[var(--color-border)] bg-white/76 p-5">
          <p className="eyebrow text-[var(--color-soft-ink)]">Artifacts</p>
          <div className="mt-4 space-y-3">
            {props.result.artifacts.map((artifact) => (
              <div
                key={artifact.id}
                className="rounded-[22px] border border-[var(--color-border)] bg-[var(--color-wash)]/70 px-4 py-4"
              >
                <div className="flex items-center justify-between gap-3">
                  <div className="font-semibold text-[var(--color-ink)]">{artifact.id}</div>
                  <span className="chip bg-white/70 text-[var(--color-soft-ink)]">
                    {artifact.kind}
                  </span>
                </div>
                <p className="mt-2 text-sm leading-6 text-[var(--color-soft-ink)]">
                  {artifact.summary}
                </p>
              </div>
            ))}
          </div>
        </section>
      </div>
    </section>
  );
}

function PanelHeader(props: { eyebrow: string; title: string; detail: string }) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div>
        <p className="eyebrow text-[var(--color-soft-ink)]">{props.eyebrow}</p>
        <h3 className="mt-2 font-serif text-2xl text-[var(--color-ink)]">{props.title}</h3>
      </div>
      <span className="chip bg-[var(--color-wash)] text-[var(--color-soft-ink)]">{props.detail}</span>
    </div>
  );
}

function TextField(props: {
  label: string;
  value: string;
  onInput: (value: string) => void;
  placeholder?: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="field-label">{props.label}</span>
      <input
        className="input-base"
        value={props.value}
        onInput={(event) => props.onInput((event.currentTarget as HTMLInputElement).value)}
        placeholder={props.placeholder}
      />
    </label>
  );
}

function NumberField(props: {
  label: string;
  value: number;
  min?: number;
  onInput: (value: number) => void;
}) {
  return (
    <label className="grid gap-2">
      <span className="field-label">{props.label}</span>
      <input
        className="input-base"
        type="number"
        min={props.min}
        value={props.value}
        onInput={(event) => props.onInput(Number((event.currentTarget as HTMLInputElement).value))}
      />
    </label>
  );
}

function TextAreaField(props: {
  label: string;
  value: string;
  onInput: (value: string) => void;
  placeholder?: string;
  rows?: number;
}) {
  return (
    <label className="grid gap-2">
      <span className="field-label">{props.label}</span>
      <textarea
        className="input-base min-h-[120px] resize-y"
        rows={props.rows ?? 6}
        value={props.value}
        onInput={(event) => props.onInput((event.currentTarget as HTMLTextAreaElement).value)}
        placeholder={props.placeholder}
      />
    </label>
  );
}

function SelectField(props: {
  label: string;
  value: string;
  options: Array<{ label: string; value: string }>;
  onChange: (value: string) => void;
}) {
  return (
    <label className="grid gap-2">
      <span className="field-label">{props.label}</span>
      <select
        className="input-base"
        value={props.value}
        onChange={(event) => props.onChange((event.currentTarget as HTMLSelectElement).value)}
      >
        {props.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function MetricCell(props: { label: string; value: string }) {
  return (
    <div className="rounded-[20px] bg-[var(--color-wash)]/70 px-4 py-4">
      <dt className="eyebrow text-[var(--color-soft-ink)]">{props.label}</dt>
      <dd className="mt-2 text-sm font-semibold leading-6 text-[var(--color-ink)]">{props.value}</dd>
    </div>
  );
}

function sourceTitle(source: SourceConfig): string {
  switch (source.type) {
    case "parquet":
      return "Parquet Snapshot";
    case "sqlite":
      return "SQLite Snapshot";
    case "postgres":
      return "PostgreSQL Snapshot";
  }
}

function sourceLocation(source: SourceConfig): string {
  if (source.type === "postgres") {
    return source.url;
  }
  return source.path;
}

function formatMetric(value: number | null): string {
  return value === null ? "n/a" : value.toFixed(1);
}

function correlationColor(value: number): string {
  if (value >= 0.7) {
    return "#0f766e";
  }
  if (value >= 0.35) {
    return "#2f855a";
  }
  if (value >= 0) {
    return "#7c9a3f";
  }
  if (value >= -0.35) {
    return "#b87435";
  }
  if (value >= -0.7) {
    return "#b45309";
  }
  return "#8f2d18";
}
