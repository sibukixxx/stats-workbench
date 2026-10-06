use crate::{CorrMethod, OutlierAction, OutlierMethod, Step, StepHeader, SummaryStepSpec};
use sf_core::{CorrelationStepSpec, DistStepSpec, OutlierStepSpec};

/// Build an EDA summary step.
pub fn summary<I, S>(cols: I) -> Step
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    Step::EdaSummary(SummaryStepSpec {
        header: StepHeader::default(),
        cols: cols.into_iter().map(Into::into).collect(),
    })
}

/// Build an EDA correlation step.
pub fn correlation<I, S>(cols: I, method: CorrMethod) -> Step
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    Step::EdaCorrelation(CorrelationStepSpec {
        header: StepHeader::default(),
        cols: cols.into_iter().map(Into::into).collect(),
        method,
    })
}

/// Build an EDA distribution step.
pub fn distribution(col: impl Into<String>, bins: u32) -> Step {
    Step::EdaDist(DistStepSpec {
        header: StepHeader::default(),
        col: col.into(),
        bins,
    })
}

/// Build an outlier-detection step.
pub fn outlier<I, S>(cols: I) -> Step
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    Step::OutlierDetect(OutlierStepSpec {
        header: StepHeader::default(),
        cols: cols.into_iter().map(Into::into).collect(),
        method: OutlierMethod::Iqr,
        action: OutlierAction::Flag,
    })
}
