use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Stats ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsArtifact {
    pub kind: StatsKind,
    pub values: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsKind {
    Summary,
    Correlation,
    Distribution,
    OutlierDetection,
    Regression,
    RegressionDiagnostics,
    TTest,
    Anova,
    ChiSquareTest,
    ConfidenceInterval,
    ProportionTest,
    VarianceTest,
    Adf,
    Decompose,
    ArimaForecast,
    BayesianSummary,
}

// ── Plot ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlotArtifact {
    pub title: String,
    pub svg: String,
}

// ── Forecast ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub index: String,
    pub yhat: f64,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastArtifact {
    pub model: String,
    pub points: Vec<ForecastPoint>,
}

// ── Model ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModelArtifact {
    pub name: String,
    pub data: Vec<u8>,
}
