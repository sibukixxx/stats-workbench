use sf_core::{ArtifactStore, DatasetMeta, ExecutionMeta};

/// Pipeline execution context.
///
/// Holds the current dataset name, all artifacts, and metadata.
#[derive(Debug)]
pub struct PipelineContext {
    /// Name of the current primary dataset artifact.
    pub current: String,
    /// All intermediate and final artifacts.
    pub artifacts: ArtifactStore,
    /// Metadata about the input dataset.
    pub metadata: DatasetMeta,
    /// Metadata about the execution run.
    pub execution: ExecutionMeta,
}

impl PipelineContext {
    pub fn new() -> Self {
        Self {
            current: "source".to_string(),
            artifacts: ArtifactStore::default(),
            metadata: DatasetMeta::default(),
            execution: ExecutionMeta::default(),
        }
    }
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self::new()
    }
}
