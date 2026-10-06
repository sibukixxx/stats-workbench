use anyhow::Result;
use statforge::{
    output, source, OutputFormat, OutputSpec, RunResult, StatForge, StatForgeError, Step,
};

use crate::cli::{OutputArgs, SourceArgs};

pub async fn execute_single_step(source_args: &SourceArgs, step: Step) -> Result<RunResult> {
    let source = source::from_reference(
        &source_args.source,
        source_args.table.as_deref(),
        source_args.query.as_deref(),
    )?;

    Ok(StatForge::run_step(&source, &step).await?)
}

pub fn output_results(
    result: &RunResult,
    output_args: &OutputArgs,
    supported_formats: &[OutputFormat],
    artifact_keys: &[String],
) -> Result<()> {
    match build_output_spec(output_args, supported_formats, artifact_keys)? {
        Some(spec) => {
            output::write_output(result, &spec)?;
        }
        None => {
            output::to_terminal(result);
        }
    }

    Ok(())
}

fn build_output_spec(
    output_args: &OutputArgs,
    supported_formats: &[OutputFormat],
    artifact_keys: &[String],
) -> Result<Option<OutputSpec>> {
    match (&output_args.format, &output_args.out) {
        (None, None) => Ok(None),
        _ => {
            let output_format: OutputFormat =
                output_args.format.as_deref().unwrap_or("json").parse()?;

            if !supported_formats.contains(&output_format) {
                return Err(StatForgeError::Validation(format!(
                    "Unsupported output format: '{}'. Supported formats: {}",
                    output_format,
                    format_names(supported_formats)
                ))
                .into());
            }

            Ok(Some(OutputSpec {
                terminal: false,
                format: Some(output_format),
                path: output_args.out.clone(),
                report: None,
                artifacts: artifact_keys.to_vec(),
            }))
        }
    }
}

fn format_names(formats: &[OutputFormat]) -> String {
    formats
        .iter()
        .map(|format| format.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
