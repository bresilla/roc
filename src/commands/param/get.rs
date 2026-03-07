use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::{json, Value};

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{
    format_parameter_value_for_display, parameter_type_to_string, ParamClientContext,
};
use rclrs::vendor::rcl_interfaces::msg::{ParameterType, ParameterValue};

fn parameter_value_to_json(value: &ParameterValue) -> Value {
    match value.type_ {
        ParameterType::PARAMETER_NOT_SET => Value::Null,
        ParameterType::PARAMETER_BOOL => json!(value.bool_value),
        ParameterType::PARAMETER_INTEGER => json!(value.integer_value),
        ParameterType::PARAMETER_DOUBLE => json!(value.double_value),
        ParameterType::PARAMETER_STRING => json!(value.string_value),
        ParameterType::PARAMETER_BYTE_ARRAY => json!(value.byte_array_value),
        ParameterType::PARAMETER_BOOL_ARRAY => json!(value.bool_array_value),
        ParameterType::PARAMETER_INTEGER_ARRAY => json!(value.integer_array_value),
        ParameterType::PARAMETER_DOUBLE_ARRAY => json!(value.double_array_value),
        ParameterType::PARAMETER_STRING_ARRAY => json!(value.string_array_value),
        _ => Value::Null,
    }
}

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;
    let param_name = matches
        .get_one::<String>("param_name")
        .ok_or_else(|| anyhow!("param_name is required"))?;

    if matches.get_flag("include_hidden_nodes") {
        eprintln!("Note: --include-hidden-nodes is not yet supported in native mode");
    }
    if common_args.use_sim_time {
        eprintln!("Note: --use-sim-time is not yet supported in native mode");
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

    let hide_type = matches.get_flag("hide_type");
    let node_fqn = ParamClientContext::node_fqn(node_name);
    let mut ctx = ParamClientContext::new()?;

    let response = ctx.get_parameters(&node_fqn, vec![param_name.to_string()])?;
    let value = response
        .values
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No value returned for parameter '{}'", param_name))?;

    let display_value = format_parameter_value_for_display(&value, hide_type);
    let type_name = parameter_type_to_string(value.type_);

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Parameter");
            blocks::print_field("Node", &node_fqn);
            blocks::print_field("Name", param_name);
            if !hide_type {
                blocks::print_field("Type", type_name);
            }
            blocks::print_field("Value", format_parameter_value_for_display(&value, true));
        }
        output::OutputMode::Plain => println!("{display_value}"),
        output::OutputMode::Json => {
            output::print_json(&json!({
                "node": node_fqn,
                "name": param_name,
                "type": type_name,
                "value": parameter_value_to_json(&value),
                "display_value": format_parameter_value_for_display(&value, true),
            }))?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
