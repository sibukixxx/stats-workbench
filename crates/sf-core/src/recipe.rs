use crate::{SfError, SfResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ── Top-level Recipe ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub source: SourceSpec,
    #[serde(default)]
    pub steps: Vec<StepSpec>,
    pub output: Option<OutputSpec>,
}

// ── SourceSpec (tagged enum) ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceSpec {
    Parquet {
        path: String,
    },
    Sqlite {
        path: String,
        table: Option<String>,
        query: Option<String>,
    },
    Postgres {
        url: String,
        table: Option<String>,
        query: Option<String>,
        filter: Option<String>,
    },
}

impl SourceSpec {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Parquet { .. } => "parquet",
            Self::Sqlite { .. } => "sqlite",
            Self::Postgres { .. } => "postgres",
        }
    }

    pub fn location(&self) -> &str {
        match self {
            Self::Parquet { path } | Self::Sqlite { path, .. } => path,
            Self::Postgres { url, .. } => url,
        }
    }

    pub fn validate(&self) -> SfResult<()> {
        match self {
            Self::Sqlite { table, query, .. } | Self::Postgres { table, query, .. } => {
                if table.is_none() && query.is_none() {
                    return Err(SfError::Validation(
                        "Database source requires either 'table' or 'query'".into(),
                    ));
                }
            }
            Self::Parquet { .. } => {}
        }

        Ok(())
    }
}

// ── StepSpec (tagged enum) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepSpec {
    #[serde(rename = "eda.summary")]
    EdaSummary(SummaryStepSpec),

    #[serde(rename = "eda.correlation")]
    EdaCorrelation(CorrelationStepSpec),

    #[serde(rename = "eda.dist")]
    EdaDist(DistStepSpec),

    #[serde(rename = "stats.outlier_detect")]
    OutlierDetect(OutlierStepSpec),

    #[serde(rename = "model.regress")]
    ModelRegress(RegressStepSpec),

    #[serde(rename = "model.diagnose")]
    ModelDiagnose(DiagnoseStepSpec),

    #[serde(rename = "ts.adf")]
    TsAdf(AdfStepSpec),

    #[serde(rename = "ts.decompose")]
    TsDecompose(DecomposeStepSpec),

    #[serde(rename = "ts.forecast")]
    TsForecast(ForecastStepSpec),

    #[serde(rename = "infer.ttest_one_sample")]
    InferTTestOneSample(TTestOneSampleStepSpec),

    #[serde(rename = "infer.ttest_two_sample")]
    InferTTestTwoSample(TTestTwoSampleStepSpec),

    #[serde(rename = "infer.mean_ci")]
    InferMeanCi(MeanCiStepSpec),

    #[serde(rename = "infer.proportion_test")]
    InferProportionTest(ProportionTestStepSpec),

    #[serde(rename = "infer.proportion_ci")]
    InferProportionCi(ProportionCiStepSpec),

    #[serde(rename = "infer.variance_test")]
    InferVarianceTest(VarianceTestStepSpec),

    #[serde(rename = "infer.variance_ci")]
    InferVarianceCi(VarianceCiStepSpec),

    #[serde(rename = "infer.chisq_goodness")]
    InferChiSqGoodness(ChiSqGoodnessStepSpec),

    #[serde(rename = "infer.chisq_independence")]
    InferChiSqIndependence(ChiSqIndependenceStepSpec),
}

impl StepSpec {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::EdaSummary(_) => "eda.summary",
            Self::EdaCorrelation(_) => "eda.correlation",
            Self::EdaDist(_) => "eda.dist",
            Self::OutlierDetect(_) => "stats.outlier_detect",
            Self::ModelRegress(_) => "model.regress",
            Self::ModelDiagnose(_) => "model.diagnose",
            Self::TsAdf(_) => "ts.adf",
            Self::TsDecompose(_) => "ts.decompose",
            Self::TsForecast(_) => "ts.forecast",
            Self::InferTTestOneSample(_) => "infer.ttest_one_sample",
            Self::InferTTestTwoSample(_) => "infer.ttest_two_sample",
            Self::InferMeanCi(_) => "infer.mean_ci",
            Self::InferProportionTest(_) => "infer.proportion_test",
            Self::InferProportionCi(_) => "infer.proportion_ci",
            Self::InferVarianceTest(_) => "infer.variance_test",
            Self::InferVarianceCi(_) => "infer.variance_ci",
            Self::InferChiSqGoodness(_) => "infer.chisq_goodness",
            Self::InferChiSqIndependence(_) => "infer.chisq_independence",
        }
    }

    pub fn header(&self) -> &StepHeader {
        match self {
            Self::EdaSummary(spec) => &spec.header,
            Self::EdaCorrelation(spec) => &spec.header,
            Self::EdaDist(spec) => &spec.header,
            Self::OutlierDetect(spec) => &spec.header,
            Self::ModelRegress(spec) => &spec.header,
            Self::ModelDiagnose(spec) => &spec.header,
            Self::TsAdf(spec) => &spec.header,
            Self::TsDecompose(spec) => &spec.header,
            Self::TsForecast(spec) => &spec.header,
            Self::InferTTestOneSample(spec) => &spec.header,
            Self::InferTTestTwoSample(spec) => &spec.header,
            Self::InferMeanCi(spec) => &spec.header,
            Self::InferProportionTest(spec) => &spec.header,
            Self::InferProportionCi(spec) => &spec.header,
            Self::InferVarianceTest(spec) => &spec.header,
            Self::InferVarianceCi(spec) => &spec.header,
            Self::InferChiSqGoodness(spec) => &spec.header,
            Self::InferChiSqIndependence(spec) => &spec.header,
        }
    }

    pub fn output_name(&self, step_index: usize) -> String {
        if let Some(output) = &self.header().output {
            output.clone()
        } else if let Some(id) = &self.header().id {
            id.clone()
        } else {
            format!(
                "step_{:02}_{}",
                step_index + 1,
                self.default_output_prefix()
            )
        }
    }

    pub fn validate(&self, step_number: usize) -> SfResult<()> {
        match self {
            Self::EdaSummary(_) => Ok(()),
            Self::EdaCorrelation(spec) => {
                if spec.cols.len() < 2 {
                    Err(SfError::Validation(format!(
                        "Step {}: correlation requires at least 2 columns",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::EdaDist(spec) => {
                if spec.bins == 0 {
                    Err(SfError::Validation(format!(
                        "Step {}: bins must be > 0",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::OutlierDetect(spec) => {
                if spec.cols.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: outlier_detect requires at least one column",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::ModelRegress(spec) => {
                if spec.target.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: model.regress requires a target column",
                        step_number
                    )))
                } else if spec.predictors.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: model.regress requires at least one predictor column",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::ModelDiagnose(spec) => {
                if spec.target.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: model.diagnose requires a target column",
                        step_number
                    )))
                } else if spec.predictors.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: model.diagnose requires at least one predictor column",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::TsAdf(spec) => {
                if spec.time_col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: ts.adf requires time_col",
                        step_number
                    )))
                } else if spec.value_col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: ts.adf requires value_col",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::TsDecompose(spec) => {
                if spec.time_col.is_empty() || spec.value_col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: ts.decompose requires time_col and value_col",
                        step_number
                    )))
                } else if spec.period == 0 {
                    Err(SfError::Validation(format!(
                        "Step {}: ts.decompose requires period > 0",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::TsForecast(spec) => {
                if spec.time_col.is_empty() || spec.value_col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: ts.forecast requires time_col and value_col",
                        step_number
                    )))
                } else if spec.horizon == 0 {
                    Err(SfError::Validation(format!(
                        "Step {}: ts.forecast requires horizon > 0",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::InferTTestOneSample(spec) => {
                if spec.col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: ttest_one_sample requires a column",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::InferTTestTwoSample(spec) => {
                if spec.col.is_empty() || spec.group_col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: ttest_two_sample requires col and group_col",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::InferMeanCi(spec) => {
                if spec.col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: mean_ci requires a column",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::InferProportionTest(_) => Ok(()),
            Self::InferProportionCi(_) => Ok(()),
            Self::InferVarianceTest(spec) => {
                if spec.col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: variance_test requires a column",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::InferVarianceCi(spec) => {
                if spec.col.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: variance_ci requires a column",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::InferChiSqGoodness(spec) => {
                if spec.observed.len() < 2 {
                    Err(SfError::Validation(format!(
                        "Step {}: chisq_goodness requires at least 2 categories",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
            Self::InferChiSqIndependence(spec) => {
                if spec.col1.is_empty() || spec.col2.is_empty() {
                    Err(SfError::Validation(format!(
                        "Step {}: chisq_independence requires col1 and col2",
                        step_number
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn default_output_prefix(&self) -> &'static str {
        match self {
            Self::EdaSummary(_) => "eda_summary",
            Self::EdaCorrelation(_) => "eda_correlation",
            Self::EdaDist(_) => "eda_dist",
            Self::OutlierDetect(_) => "outlier_detect",
            Self::ModelRegress(_) => "model_regress",
            Self::ModelDiagnose(_) => "model_diagnose",
            Self::TsAdf(_) => "ts_adf",
            Self::TsDecompose(_) => "ts_decompose",
            Self::TsForecast(_) => "ts_forecast",
            Self::InferTTestOneSample(_) => "infer_ttest_1",
            Self::InferTTestTwoSample(_) => "infer_ttest_2",
            Self::InferMeanCi(_) => "infer_mean_ci",
            Self::InferProportionTest(_) => "infer_proportion_test",
            Self::InferProportionCi(_) => "infer_proportion_ci",
            Self::InferVarianceTest(_) => "infer_variance_test",
            Self::InferVarianceCi(_) => "infer_variance_ci",
            Self::InferChiSqGoodness(_) => "infer_chisq_goodness",
            Self::InferChiSqIndependence(_) => "infer_chisq_independence",
        }
    }
}

// ── StepHeader (common fields) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepHeader {
    pub id: Option<String>,
    pub input: Option<String>,
    pub output: Option<String>,
}

// ── Individual StepSpecs ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    #[serde(default)]
    pub cols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub cols: Vec<String>,
    #[serde(default = "default_corr_method")]
    pub method: CorrMethod,
}

fn default_corr_method() -> CorrMethod {
    CorrMethod::Pearson
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorrMethod {
    Pearson,
    Spearman,
}

impl fmt::Display for CorrMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Pearson => "pearson",
            Self::Spearman => "spearman",
        };
        f.write_str(value)
    }
}

impl FromStr for CorrMethod {
    type Err = SfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pearson" => Ok(Self::Pearson),
            "spearman" => Ok(Self::Spearman),
            other => Err(SfError::Validation(format!(
                "Unknown correlation method: '{}'. Use 'pearson' or 'spearman'.",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub col: String,
    #[serde(default = "default_bins")]
    pub bins: u32,
}

fn default_bins() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub cols: Vec<String>,
    #[serde(default)]
    pub method: OutlierMethod,
    #[serde(default)]
    pub action: OutlierAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutlierMethod {
    #[default]
    Iqr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutlierAction {
    #[default]
    Flag,
}

// ── Model step specs ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub target: String,
    pub predictors: Vec<String>,
    #[serde(default = "default_confidence_level")]
    pub confidence_level: f64,
}

fn default_confidence_level() -> f64 {
    0.95
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnoseStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub target: String,
    pub predictors: Vec<String>,
}

// ── Time-series StepSpecs ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub time_col: String,
    pub value_col: String,
    #[serde(default = "default_adf_max_lag")]
    pub max_lag: usize,
}

fn default_adf_max_lag() -> usize {
    12
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub time_col: String,
    pub value_col: String,
    pub period: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub time_col: String,
    pub value_col: String,
    #[serde(default = "default_arima_order")]
    pub order: [usize; 3],
    #[serde(default = "default_horizon")]
    pub horizon: usize,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_arima_order() -> [usize; 3] {
    [1, 1, 1]
}

fn default_horizon() -> usize {
    10
}

fn default_confidence() -> f64 {
    0.95
}

// ── Infer StepSpecs ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTestOneSampleStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub col: String,
    #[serde(default)]
    pub mu0: f64,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub alternative: AlternativeSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTestTwoSampleStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub col: String,
    pub group_col: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub alternative: AlternativeSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanCiStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub col: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProportionTestStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub n: u64,
    pub successes: u64,
    #[serde(default = "default_p0")]
    pub p0: f64,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub alternative: AlternativeSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProportionCiStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub n: u64,
    pub successes: u64,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceTestStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub col: String,
    pub sigma2_0: f64,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub alternative: AlternativeSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceCiStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub col: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChiSqGoodnessStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub observed: Vec<f64>,
    pub expected: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChiSqIndependenceStepSpec {
    #[serde(flatten)]
    pub header: StepHeader,
    pub col1: String,
    pub col2: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlternativeSpec {
    #[default]
    TwoSided,
    Less,
    Greater,
}

fn default_p0() -> f64 {
    0.5
}


// ── OutputSpec ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    #[serde(default)]
    pub terminal: bool,
    pub format: Option<OutputFormat>,
    pub path: Option<String>,
    pub report: Option<String>,
    #[serde(default)]
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Json,
    Parquet,
    Svg,
    Html,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Json => "json",
            Self::Parquet => "parquet",
            Self::Svg => "svg",
            Self::Html => "html",
        };
        f.write_str(value)
    }
}

impl FromStr for OutputFormat {
    type Err = SfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "parquet" => Ok(Self::Parquet),
            "svg" => Ok(Self::Svg),
            "html" => Ok(Self::Html),
            other => Err(SfError::Validation(format!(
                "Unsupported output format: '{}'",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_spec_validates_database_requirements() {
        let invalid = SourceSpec::Sqlite {
            path: "test.db".into(),
            table: None,
            query: None,
        };

        let error = invalid.validate().unwrap_err();
        assert_eq!(
            error.to_string(),
            "validation error: Database source requires either 'table' or 'query'"
        );
    }

    #[test]
    fn step_spec_prefers_output_name_over_id() {
        let step = StepSpec::EdaSummary(SummaryStepSpec {
            header: StepHeader {
                id: Some("summary_main".into()),
                input: None,
                output: Some("custom_output".into()),
            },
            cols: vec!["price".into()],
        });

        assert_eq!(step.output_name(0), "custom_output");
    }

    #[test]
    fn output_format_parses_case_insensitively() {
        assert_eq!(
            "PARQUET".parse::<OutputFormat>().unwrap(),
            OutputFormat::Parquet
        );
    }

    #[test]
    fn correlation_method_rejects_unknown_values() {
        let result = "kendall".parse::<CorrMethod>();
        assert!(result.is_err());
    }
}
