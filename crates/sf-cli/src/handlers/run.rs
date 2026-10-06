use anyhow::Result;
use statforge::{output, StatForge};

use crate::cli::RunCommand;

pub async fn handle_run(args: RunCommand) -> Result<()> {
    let recipe = StatForge::load_recipe(&args.recipe)?;
    let result = StatForge::run_recipe_file(&args.recipe).await?;

    // Handle output
    if let Some(ref output_spec) = recipe.output {
        if output_spec.terminal {
            output::to_terminal(&result);
        }
        if output_spec.format.is_some() || output_spec.report.is_some() {
            output::write_output(&result, output_spec)?;
        }
    } else {
        // No output spec → print to terminal
        output::to_terminal(&result);
    }

    Ok(())
}
