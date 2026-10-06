use serde::{Deserialize, Serialize};

/// Metadata about the input dataset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatasetMeta {
    pub source_type: String,
    pub source_path: String,
    pub row_count: Option<u64>,
    pub col_count: Option<u64>,
    pub columns: Vec<String>,
}

/// Metadata about the execution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMeta {
    pub tool: String,
    pub version: String,
    pub recipe_path: Option<String>,
    pub started_at: String,
    pub steps_executed: u32,
}

impl Default for ExecutionMeta {
    fn default() -> Self {
        Self {
            tool: "Stats Workbench".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            recipe_path: None,
            started_at: String::new(),
            steps_executed: 0,
        }
    }
}
