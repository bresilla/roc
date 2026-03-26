use crate::commands::cli::handle_anyhow_result;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::ParamClientContext;

use rclrs::vendor::rcl_interfaces::msg::{ParameterType, ParameterValue};

fn node_fqn_to_filename(node_fqn: &str) -> String {
    let trimmed = node_fqn.trim_matches('/');
    if trimmed.is_empty() {
        return "node.yaml".to_string();
    }
    let replaced = trimmed.replace('/', "_");
    format!("{}.yaml", replaced)
}

fn parameter_value_to_yaml_value(v: &ParameterValue) -> Option<Value> {
    match v.type_ {
        ParameterType::PARAMETER_NOT_SET => None,
        ParameterType::PARAMETER_BOOL => Some(Value::from(v.bool_value)),
        ParameterType::PARAMETER_INTEGER => Some(Value::from(v.integer_value)),
        ParameterType::PARAMETER_DOUBLE => Some(Value::from(v.double_value)),
        ParameterType::PARAMETER_STRING => Some(Value::from(v.string_value.clone())),
        ParameterType::PARAMETER_BYTE_ARRAY => Some(Value::Sequence(
            v.byte_array_value.iter().map(|b| Value::from(*b)).collect(),
        )),
        ParameterType::PARAMETER_BOOL_ARRAY => Some(Value::Sequence(
            v.bool_array_value.iter().map(|b| Value::from(*b)).collect(),
        )),
        ParameterType::PARAMETER_INTEGER_ARRAY => Some(Value::Sequence(
            v.integer_array_value
                .iter()
                .map(|i| Value::from(*i))
                .collect(),
        )),
        ParameterType::PARAMETER_DOUBLE_ARRAY => Some(Value::Sequence(
            v.double_array_value
                .iter()
                .map(|f| Value::from(*f))
                .collect(),
        )),
        ParameterType::PARAMETER_STRING_ARRAY => Some(Value::Sequence(
            v.string_array_value
                .iter()
                .cloned()
                .map(Value::from)
                .collect(),
        )),
        _ => None,
    }
}

fn insert_nested(mapping: &mut Mapping, dotted_name: &str, value: Value) {
    let mut current = mapping;
    let parts: Vec<&str> = dotted_name.split('.').collect();
    for (idx, part) in parts.iter().enumerate() {
        let key = Value::String(part.to_string());
        if idx == parts.len() - 1 {
            current.insert(key, value);
            return;
        }

        // Ensure the intermediate namespace exists as a mapping.
        let next = {
            let entry = current
                .entry(key.clone())
                .or_insert_with(|| Value::Mapping(Mapping::new()));
            if !matches!(entry, Value::Mapping(_)) {
                *entry = Value::Mapping(Mapping::new());
            }
            match entry {
                Value::Mapping(m) => m,
                _ => return,
            }
        };

        current = next;
    }
}

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;

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

    let node_fqn = ParamClientContext::node_fqn(node_name);
    let mut ctx = ParamClientContext::new()?;

    let output_dir: Option<PathBuf> = matches
        .get_many::<String>("output_dir")
        .and_then(|mut it| it.next())
        .map(PathBuf::from);

    let list = ctx.list_parameters(&node_fqn, Vec::new())?;
    let mut names: Vec<String> = list.result.names.into_iter().collect();
    names.sort();

    let values = ctx.get_parameters(&node_fqn, names.clone())?;
    if values.values.len() != names.len() {
        return Err(anyhow!(
            "Mismatched response: expected {} values, got {}",
            names.len(),
            values.values.len()
        ));
    }

    let mut ros_params = Mapping::new();
    for (name, v) in names.into_iter().zip(values.values.into_iter()) {
        let Some(yaml_value) = parameter_value_to_yaml_value(&v) else {
            continue;
        };
        insert_nested(&mut ros_params, &name, yaml_value);
    }

    let mut node_entry = Mapping::new();
    node_entry.insert(
        Value::String("ros__parameters".to_string()),
        Value::Mapping(ros_params),
    );
    let mut root = Mapping::new();
    root.insert(Value::String(node_fqn.clone()), Value::Mapping(node_entry));

    let yaml = serde_yaml::to_string(&Value::Mapping(root))?;

    let out_path = if let Some(dir) = output_dir {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        dir.join(node_fqn_to_filename(&node_fqn))
    } else {
        Path::new(&node_fqn_to_filename(&node_fqn)).to_path_buf()
    };

    fs::write(&out_path, yaml)?;
    println!("Wrote parameter file: {}", out_path.display());
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}
