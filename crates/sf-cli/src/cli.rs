use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "stats-workbench",
    version,
    about = "Stats Workbench の分析メニューです",
    long_about = "Stats Workbench の分析メニューです。\nParquet / SQLite / PostgreSQL を対象に、探索的データ分析や統計処理、レシピ実行を行います。"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(
        about = "探索的データ分析メニュー",
        long_about = "探索的データ分析メニューです。\n要約統計、相関、分布を確認して、データの全体像や傾向を素早く把握します。"
    )]
    Eda(EdaCommand),
    #[command(
        about = "統計処理メニュー",
        long_about = "統計処理メニューです。\n外れ値検出などの統計処理を実行し、分析時に確認したい異常値や特徴を洗い出します。"
    )]
    Stats(StatsCommand),
    /// Linear model fitting and diagnostics
    Model(ModelCommand),
    /// Time series analysis
    Ts(TsCommand),
    /// Inferential statistics (hypothesis tests & confidence intervals)
    Infer(InferCommand),
    #[command(
        about = "レシピ実行メニュー",
        long_about = "レシピ実行メニューです。\nTOML で定義した分析手順をまとめて実行し、再現可能なパイプラインとして利用します。"
    )]
    Run(RunCommand),
}

// ── EDA ────────────────────────────────────────────

#[derive(Parser, Debug)]
pub struct EdaCommand {
    #[command(subcommand)]
    pub command: EdaSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum EdaSubcommand {
    #[command(
        about = "要約統計画面",
        long_about = "要約統計画面です。\n列ごとの件数、平均、最小値、最大値などを確認して、データの基本的な傾向を把握します。"
    )]
    Summary(EdaSummaryArgs),
    #[command(
        about = "相関分析画面",
        long_about = "相関分析画面です。\n複数列の相関係数を算出して、項目同士の関係性の強さや方向を確認します。"
    )]
    Corr(EdaCorrArgs),
    #[command(
        about = "分布確認画面",
        long_about = "分布確認画面です。\n指定した列のヒストグラムを計算して、値のばらつきや偏りを確認します。"
    )]
    Dist(EdaDistArgs),
}

// ── Stats ──────────────────────────────────────────

#[derive(Parser, Debug)]
pub struct StatsCommand {
    #[command(subcommand)]
    pub command: StatsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum StatsSubcommand {
    #[command(
        about = "外れ値検出画面",
        long_about = "外れ値検出画面です。\nIQR 法を使って外れ値候補を抽出し、異常な値や確認が必要なデータを見つけます。"
    )]
    Outlier(OutlierArgs),
}

// ── Infer ──────────────────────────────────────────

#[derive(Parser, Debug)]
pub struct InferCommand {
    #[command(subcommand)]
    pub command: InferSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum InferSubcommand {
    /// One-sample t-test
    Ttest1(TTest1Args),
    /// Two-sample t-test (Welch's)
    Ttest2(TTest2Args),
    /// Confidence interval for population mean
    MeanCi(MeanCiArgs),
    /// Proportion z-test
    PropTest(PropTestArgs),
    /// Confidence interval for population proportion
    PropCi(PropCiArgs),
    /// Chi-squared test for population variance
    VarTest(VarTestArgs),
    /// Confidence interval for population variance
    VarCi(VarCiArgs),
    /// Chi-squared goodness-of-fit test
    ChisqGoodness(ChiSqGoodnessArgs),
    /// Chi-squared test of independence
    ChisqIndependence(ChiSqIndependenceArgs),
}

// ── Run ────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct RunCommand {
    /// Path to the recipe TOML file
    #[arg(long)]
    pub recipe: String,
}

// ── Common source args ─────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct SourceArgs {
    /// Data source (path.parquet, sqlite://path, pg://host/db)
    #[arg(long)]
    pub source: String,

    /// Table name (required for database sources)
    #[arg(long)]
    pub table: Option<String>,

    /// Custom SQL query
    #[arg(long)]
    pub query: Option<String>,
}

// ── Common output args ─────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct OutputArgs {
    /// Output file path (omit for terminal)
    #[arg(long)]
    pub out: Option<String>,

    /// Output format. EDA: json or parquet. stats outlier: json. Default is terminal.
    #[arg(long)]
    pub format: Option<String>,
}

// ── EDA args ───────────────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct EdaSummaryArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Columns to analyze (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub cols: Vec<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct EdaCorrArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Columns to correlate (comma-separated, min 2)
    #[arg(long, value_delimiter = ',')]
    pub cols: Vec<String>,

    /// Correlation method: pearson or spearman
    #[arg(long, default_value = "pearson")]
    pub method: String,
}

#[derive(clap::Args, Debug, Clone)]
pub struct EdaDistArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Column to compute distribution for
    #[arg(long)]
    pub col: String,

    /// Number of histogram bins
    #[arg(long, default_value_t = 10)]
    pub bins: u32,
}

// ── Stats args ─────────────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct OutlierArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Columns to check for outliers (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub cols: Vec<String>,
}


// ── Model ──────────────────────────────────────────

#[derive(Parser, Debug)]
pub struct ModelCommand {
    #[command(subcommand)]
    pub command: ModelSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ModelSubcommand {
    /// Fit an OLS regression model
    Regress(RegressArgs),
    /// Run regression diagnostics
    Diagnose(DiagnoseArgs),
}

// ── Model args ────────────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct RegressArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Target (dependent) column
    #[arg(long)]
    pub target: String,

    /// Predictor columns (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub predictors: Vec<String>,

    /// Confidence level for intervals (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,
}

#[derive(clap::Args, Debug, Clone)]
pub struct DiagnoseArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Target (dependent) column
    #[arg(long)]
    pub target: String,

    /// Predictor columns (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub predictors: Vec<String>,
}

// ── Time series ───────────────────────────────────

#[derive(Parser, Debug)]
pub struct TsCommand {
    #[command(subcommand)]
    pub command: TsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum TsSubcommand {
    /// Augmented Dickey-Fuller stationarity test
    Adf(TsAdfArgs),
    /// Seasonal decomposition (trend + seasonal + residual)
    Decompose(TsDecomposeArgs),
    /// ARIMA forecast
    Forecast(TsForecastArgs),
}

// ── Time series args ──────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct TsAdfArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Time column name
    #[arg(long)]
    pub time_col: String,

    /// Value column name
    #[arg(long)]
    pub value_col: String,

    /// Maximum lag for ADF test
    #[arg(long, default_value_t = 12)]
    pub max_lag: usize,
}

#[derive(clap::Args, Debug, Clone)]
pub struct TsDecomposeArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Time column name
    #[arg(long)]
    pub time_col: String,

    /// Value column name
    #[arg(long)]
    pub value_col: String,

    /// Seasonal period
    #[arg(long)]
    pub period: usize,
}

#[derive(clap::Args, Debug, Clone)]
pub struct TsForecastArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Time column name
    #[arg(long)]
    pub time_col: String,

    /// Value column name
    #[arg(long)]
    pub value_col: String,

    /// ARIMA order as p,d,q (comma-separated)
    #[arg(long, default_value = "1,1,1", value_delimiter = ',')]
    pub order: Vec<usize>,

    /// Number of forecast periods
    #[arg(long, default_value_t = 10)]
    pub horizon: usize,

    /// Confidence level for prediction intervals
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,
}

// ── Infer args ────────────────────────────────────

#[derive(clap::Args, Debug, Clone)]
pub struct TTest1Args {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Column to test
    #[arg(long)]
    pub col: String,

    /// Null hypothesis mean (default: 0)
    #[arg(long, default_value_t = 0.0)]
    pub mu0: f64,

    /// Confidence level (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,

    /// Alternative hypothesis: two_sided, less, greater
    #[arg(long, default_value = "two_sided")]
    pub alternative: String,
}

#[derive(clap::Args, Debug, Clone)]
pub struct TTest2Args {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Value column to test
    #[arg(long)]
    pub col: String,

    /// Grouping column (must have exactly 2 groups)
    #[arg(long)]
    pub group_col: String,

    /// Confidence level (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,

    /// Alternative hypothesis: two_sided, less, greater
    #[arg(long, default_value = "two_sided")]
    pub alternative: String,
}

#[derive(clap::Args, Debug, Clone)]
pub struct MeanCiArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Column to compute CI for
    #[arg(long)]
    pub col: String,

    /// Confidence level (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,
}

#[derive(clap::Args, Debug, Clone)]
pub struct PropTestArgs {
    #[command(flatten)]
    pub output: OutputArgs,

    /// Sample size
    #[arg(long)]
    pub n: u64,

    /// Number of successes
    #[arg(long)]
    pub successes: u64,

    /// Null hypothesis proportion (default: 0.5)
    #[arg(long, default_value_t = 0.5)]
    pub p0: f64,

    /// Confidence level (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,

    /// Alternative hypothesis: two_sided, less, greater
    #[arg(long, default_value = "two_sided")]
    pub alternative: String,
}

#[derive(clap::Args, Debug, Clone)]
pub struct PropCiArgs {
    #[command(flatten)]
    pub output: OutputArgs,

    /// Sample size
    #[arg(long)]
    pub n: u64,

    /// Number of successes
    #[arg(long)]
    pub successes: u64,

    /// Confidence level (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,
}

#[derive(clap::Args, Debug, Clone)]
pub struct VarTestArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Column to test
    #[arg(long)]
    pub col: String,

    /// Null hypothesis variance
    #[arg(long)]
    pub sigma2_0: f64,

    /// Confidence level (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,

    /// Alternative hypothesis: two_sided, less, greater
    #[arg(long, default_value = "two_sided")]
    pub alternative: String,
}

#[derive(clap::Args, Debug, Clone)]
pub struct VarCiArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// Column to compute CI for
    #[arg(long)]
    pub col: String,

    /// Confidence level (default: 0.95)
    #[arg(long, default_value_t = 0.95)]
    pub confidence: f64,
}

#[derive(clap::Args, Debug, Clone)]
pub struct ChiSqGoodnessArgs {
    #[command(flatten)]
    pub output: OutputArgs,

    /// Observed frequencies (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub observed: Vec<f64>,

    /// Expected frequencies (comma-separated, optional — defaults to uniform)
    #[arg(long, value_delimiter = ',')]
    pub expected: Vec<f64>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct ChiSqIndependenceArgs {
    #[command(flatten)]
    pub source: SourceArgs,
    #[command(flatten)]
    pub output: OutputArgs,

    /// First categorical column
    #[arg(long)]
    pub col1: String,

    /// Second categorical column
    #[arg(long)]
    pub col2: String,
}
