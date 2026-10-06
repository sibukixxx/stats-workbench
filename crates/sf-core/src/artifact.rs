use crate::dataset::DatasetRef;
use crate::error::{SfError, SfResult};
use crate::types::{ForecastArtifact, ModelArtifact, PlotArtifact, StatsArtifact};
use polars::prelude::DataFrame;
use std::collections::HashMap;

/// A single analysis artifact produced by a step.
#[derive(Debug, Clone)]
pub enum Artifact {
    Dataset(DatasetRef),
    Stats(StatsArtifact),
    Plot(PlotArtifact),
    Forecast(ForecastArtifact),
    Model(ModelArtifact),
    Table(DataFrame),
    Text(String),
}

/// Named store for intermediate and final artifacts.
#[derive(Debug, Default)]
pub struct ArtifactStore {
    inner: HashMap<String, Artifact>,
    /// Insertion order for deterministic iteration.
    order: Vec<String>,
}

impl ArtifactStore {
    pub fn insert(&mut self, key: impl Into<String>, value: Artifact) -> Option<Artifact> {
        let key = key.into();
        if !self.inner.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&Artifact> {
        self.inner.get(key)
    }

    pub fn require(&self, key: &str) -> SfResult<&Artifact> {
        self.get(key)
            .ok_or_else(|| SfError::Execution(format!("artifact not found: {}", key)))
    }

    /// Iterate artifacts in insertion order.
    pub fn iter_ordered(&self) -> impl Iterator<Item = (&str, &Artifact)> {
        self.order
            .iter()
            .filter_map(|k| self.inner.get(k).map(|v| (k.as_str(), v)))
    }

    pub fn keys(&self) -> &[String] {
        &self.order
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
