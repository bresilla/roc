use crate::arguments::action::CommonActionArgs;
use crate::graph::{action_operations, RclGraphContext};
use anyhow::{anyhow, Result};
use clap::ArgMatches;

fn run_command(matches: ArgMatches, common_args: CommonActionArgs) -> Result<()> {
    if common_args.use_sim_time {
        eprintln!("Note: --use-sim-time is not applicable to graph queries");
    }
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    if let Some(spin_time_value) = common_args.spin_time {
        eprintln!(
            "Note: --spin-time {} is not yet supported in native mode",
            spin_time_value
        );
    }

    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    let mut actions = action_operations::get_action_names(&context)?;
    actions.sort();

    if matches.get_flag("count_actions") {
        println!("{}", actions.len());
        return Ok(());
    }

    let show_types = matches.get_flag("show_types");
    for name in actions {
        if show_types {
            let ty = action_operations::get_action_type(&context, &name)?
                .unwrap_or_else(|| "<unknown>".to_string());
            println!("{} [{}]", name, ty);
        } else {
            println!("{}", name);
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonActionArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
