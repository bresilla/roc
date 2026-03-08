use crate::commands::cli::handle_anyhow_result;
use crate::ui::{blocks, output};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs;

use crate::arguments::param::CommonParamArgs;
use crate::shared::param_operations::ParamClientContext;

use rclrs::vendor::rcl_interfaces::msg::{Parameter, ParameterType, ParameterValue};

fn yaml_value_to_parameter_value(v: &Value) -> Result<ParameterValue> {
    let mut out = ParameterValue::default();
    match v {
        Value::Bool(b) => {
            out.type_ = ParameterType::PARAMETER_BOOL;
            out.bool_value = *b;
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.type_ = ParameterType::PARAMETER_INTEGER;
                out.integer_value = i;
            } else if let Some(f) = n.as_f64() {
                out.type_ = ParameterType::PARAMETER_DOUBLE;
                out.double_value = f;
            } else {
                return Err(anyhow!("Unsupported numeric value '{n}'"));
            }
        }
        Value::String(s) => {
            out.type_ = ParameterType::PARAMETER_STRING;
            out.string_value = s.clone();
        }
        Value::Sequence(seq) => {
            if seq.is_empty() {
                out.type_ = ParameterType::PARAMETER_STRING_ARRAY;
                out.string_array_value = Vec::new();
                return Ok(out);
            }

            // Try bool array
            if seq.iter().all(|x| matches!(x, Value::Bool(_))) {
                out.type_ = ParameterType::PARAMETER_BOOL_ARRAY;
                out.bool_array_value = seq
                    .iter()
                    .map(|x| match x {
                        Value::Bool(b) => Ok(*b),
                        _ => Err(anyhow!("unexpected")),
                    })
                    .collect::<Result<Vec<_>>>()?;
                return Ok(out);
            }

            // Try integer array
            if seq.iter().all(|x| matches!(x, Value::Number(_)))
                && seq.iter().all(|x| x.as_i64().is_some())
            {
                out.type_ = ParameterType::PARAMETER_INTEGER_ARRAY;
                out.integer_array_value = seq
                    .iter()
                    .map(|x| x.as_i64().ok_or_else(|| anyhow!("expected integer")))
                    .collect::<Result<Vec<_>>>()?;
                return Ok(out);
            }

            // Try double array
            if seq.iter().all(|x| matches!(x, Value::Number(_))) {
                let mut doubles = Vec::with_capacity(seq.len());
                for x in seq {
                    let Some(f) = x.as_f64() else {
                        return Err(anyhow!("expected float"));
                    };
                    doubles.push(f);
                }
                out.type_ = ParameterType::PARAMETER_DOUBLE_ARRAY;
                out.double_array_value = doubles;
                return Ok(out);
            }

            // Try string array
            if seq.iter().all(|x| matches!(x, Value::String(_))) {
                out.type_ = ParameterType::PARAMETER_STRING_ARRAY;
                out.string_array_value = seq
                    .iter()
                    .map(|x| match x {
                        Value::String(s) => Ok(s.clone()),
                        _ => Err(anyhow!("unexpected")),
                    })
                    .collect::<Result<Vec<_>>>()?;
                return Ok(out);
            }

            return Err(anyhow!(
                "Unsupported YAML sequence element types for parameter value"
            ));
        }
        Value::Null => {
            out.type_ = ParameterType::PARAMETER_NOT_SET;
        }
        Value::Mapping(_) => {
            return Err(anyhow!(
                "Unsupported mapping value for parameter; expected scalar or sequence"
            ));
        }
        Value::Tagged(t) => {
            return Err(anyhow!("Unsupported tagged YAML value: {:?}", t.tag));
        }
    }
    Ok(out)
}

fn collect_params_from_yaml_mapping(
    mapping: &serde_yaml::Mapping,
    prefix: &str,
    out: &mut Vec<Parameter>,
) -> Result<()> {
    for (k, v) in mapping {
        let Some(key) = k.as_str() else {
            return Err(anyhow!("YAML mapping key must be a string"));
        };

        let full_key = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}.{key}")
        };

        match v {
            Value::Mapping(m) => {
                collect_params_from_yaml_mapping(m, &full_key, out)?;
            }
            _ => {
                let pv = yaml_value_to_parameter_value(v)?;
                out.push(Parameter {
                    name: full_key,
                    value: pv,
                });
            }
        }
    }
    Ok(())
}

fn parameters_from_yaml_mapping(mapping: &serde_yaml::Mapping) -> Result<Vec<Parameter>> {
    let mut out = Vec::new();
    collect_params_from_yaml_mapping(mapping, "", &mut out)?;
    Ok(out)
}

fn selected_ros_parameter_mapping<'a>(
    root: &'a serde_yaml::Mapping,
    node_key: &str,
) -> Result<Option<&'a serde_yaml::Mapping>> {
    let Some(node_val) = root.get(Value::String(node_key.to_string())) else {
        return Ok(None);
    };

    let node_map = node_val
        .as_mapping()
        .ok_or_else(|| anyhow!("Node entry '{node_key}' must be a mapping"))?;
    let ros_params_val = node_map
        .get(Value::String("ros__parameters".to_string()))
        .ok_or_else(|| anyhow!("Missing 'ros__parameters' under '{node_key}'"))?;

    ros_params_val
        .as_mapping()
        .map(Some)
        .ok_or_else(|| anyhow!("'ros__parameters' must be a mapping"))
}

fn selected_parameters_from_yaml(
    root: &serde_yaml::Mapping,
    node_fqn: &str,
    include_wildcard: bool,
) -> Result<Vec<Parameter>> {
    let mut merged = BTreeMap::new();

    if include_wildcard {
        if let Some(wildcard_map) = selected_ros_parameter_mapping(root, "/**")? {
            for parameter in parameters_from_yaml_mapping(wildcard_map)? {
                merged.insert(parameter.name, parameter.value);
            }
        }
    }

    if let Some(node_map) = selected_ros_parameter_mapping(root, node_fqn)? {
        for parameter in parameters_from_yaml_mapping(node_map)? {
            merged.insert(parameter.name, parameter.value);
        }
    }

    Ok(merged
        .into_iter()
        .map(|(name, value)| Parameter { name, value })
        .collect())
}

fn run_command(matches: ArgMatches, common_args: CommonParamArgs) -> Result<()> {
    let output_mode = output::OutputMode::from_matches(&matches);
    let node_name = matches
        .get_one::<String>("node_name")
        .ok_or_else(|| anyhow!("node_name is required"))?;
    let param_file = matches
        .get_one::<String>("param_name")
        .ok_or_else(|| anyhow!("param_name (parameter file) is required"))?;

    if common_args.use_sim_time {
        blocks::eprint_note("--use-sim-time is not yet supported in native mode");
    }
    if common_args.no_daemon {
        blocks::eprint_note("roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    let node_fqn = ParamClientContext::node_fqn(node_name);
    let mut ctx = ParamClientContext::new_with_spin_time(common_args.spin_time.as_deref())?;
    ctx.ensure_node_available(&node_fqn, matches.get_flag("include_hidden_nodes"))?;

    let content = fs::read_to_string(param_file)
        .map_err(|e| anyhow!("Failed to read parameter file '{}': {}", param_file, e))?;
    let yaml: Value = serde_yaml::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse YAML '{}': {}", param_file, e))?;

    let root = yaml
        .as_mapping()
        .ok_or_else(|| anyhow!("Top-level YAML must be a mapping"))?;

    let param_sets = selected_parameters_from_yaml(
        root,
        &node_fqn,
        !matches.get_flag("no_use_wildcard"),
    )?;

    if param_sets.is_empty() {
        return Err(anyhow!(
            "No parameters found for node '{}' in file '{}'",
            node_fqn,
            param_file
        ));
    }

    let loaded_count = param_sets.len();
    let response = ctx.set_parameters(&node_fqn, param_sets)?;
    let mut failures = Vec::new();
    for (idx, r) in response.results.iter().enumerate() {
        if !r.successful {
            failures.push(format!("{}: {}", idx, r.reason));
        }
    }
    if !failures.is_empty() {
        return Err(anyhow!(
            "Failed to set some parameters: {}",
            failures.join(", ")
        ));
    }

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Parameters Imported");
            blocks::print_field("Node", &node_fqn);
            blocks::print_field("File", param_file);
            blocks::print_field("Parameters", loaded_count);
        }
        output::OutputMode::Plain => {
            println!("{node_fqn}\t{param_file}\t{loaded_count}");
        }
        output::OutputMode::Json => {
            output::print_json(&serde_json::json!({
                "node": node_fqn,
                "file": param_file,
                "loaded_count": loaded_count,
                "successful": true,
            }))?;
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonParamArgs) {
    handle_anyhow_result(run_command(matches, common_args));
}

#[cfg(test)]
mod tests {
    use super::selected_parameters_from_yaml;
    use serde_yaml::Value;

    fn load_root(yaml: &str) -> serde_yaml::Mapping {
        serde_yaml::from_str::<Value>(yaml)
            .expect("valid yaml")
            .as_mapping()
            .expect("top level mapping")
            .clone()
    }

    #[test]
    fn selected_parameters_can_skip_wildcard_entries() {
        let root = load_root(
            r#"
"/**":
  ros__parameters:
    shared: 1
"/talker":
  ros__parameters:
    local: true
"#,
        );

        let parameters =
            selected_parameters_from_yaml(&root, "/talker", false).expect("parameters");
        let names = parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["local"]);
    }

    #[test]
    fn selected_parameters_merge_wildcard_and_node_specific_values() {
        let root = load_root(
            r#"
"/**":
  ros__parameters:
    shared: 1
    override_me: 2
"/talker":
  ros__parameters:
    local: true
    override_me: 42
"#,
        );

        let parameters =
            selected_parameters_from_yaml(&root, "/talker", true).expect("parameters");

        let shared = parameters
            .iter()
            .find(|parameter| parameter.name == "shared")
            .expect("shared parameter");
        assert_eq!(shared.value.integer_value, 1);

        let local = parameters
            .iter()
            .find(|parameter| parameter.name == "local")
            .expect("local parameter");
        assert!(local.value.bool_value);

        let override_me = parameters
            .iter()
            .find(|parameter| parameter.name == "override_me")
            .expect("override parameter");
        assert_eq!(override_me.value.integer_value, 42);
    }
}
