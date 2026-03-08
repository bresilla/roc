use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_json::{json, Value};

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::{
    format_parameter_value_for_display, parameter_type_to_string,
    parse_value_tokens_to_parameter_value, ParamClientContext,
};

use rclrs::vendor::rcl_interfaces::msg::{Parameter, ParameterType, ParameterValue};

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
    let value_tokens: Vec<String> = matches
        .get_many::<String>("value")
        .ok_or_else(|| anyhow!("value is required"))?
        .cloned()
        .collect();

    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    let node_fqn = ParamClientContext::node_fqn(node_name);
    let mut ctx =
        ParamClientContext::new_with_options(common_args.spin_time.as_deref(), common_args.use_sim_time)?;
    ctx.ensure_node_available(&node_fqn, matches.get_flag("include_hidden_nodes"))?;

    let value = parse_value_tokens_to_parameter_value(&value_tokens)?;
    let param = Parameter {
        name: param_name.to_string(),
        value: value.clone(),
    };

    let response = ctx.set_parameters(&node_fqn, vec![param])?;
    let first = response
        .results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No result returned by set_parameters"))?;

    if !first.successful {
        return Err(anyhow!(
            "Failed to set parameter '{}': {}",
            param_name,
            first.reason
        ));
    }

    let type_name = parameter_type_to_string(value.type_);
    let display_value = format_parameter_value_for_display(&value, true);

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Parameter Updated");
            blocks::print_field("Node", &node_fqn);
            blocks::print_field("Name", param_name);
            blocks::print_field("Type", type_name);
            blocks::print_field("Value", display_value);
        }
        output::OutputMode::Plain => println!("Set parameter {} successful", param_name),
        output::OutputMode::Json => {
            output::print_json(&json!({
                "node": node_fqn,
                "name": param_name,
                "type": type_name,
                "value": parameter_value_to_json(&value),
                "display_value": display_value,
                "successful": true,
            }))?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
