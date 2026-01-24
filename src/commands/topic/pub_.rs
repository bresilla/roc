use crate::arguments::topic::CommonTopicArgs;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use rclrs::{
    Context, CreateBasicExecutor, DynamicMessage, MessageTypeName, SimpleValueMut, ValueMut,
};
use std::thread;
use std::time::{Duration, Instant};

fn set_field_from_yaml(msg: &mut DynamicMessage, field: &str, value: &serde_yaml::Value) -> Result<()> {
    let Some(field_value) = msg.get_mut(field) else {
        return Err(anyhow!("Unknown field '{}' for type {}", field, msg.structure().type_name));
    };

    match field_value {
        ValueMut::Simple(simple) => set_simple_value_from_yaml(field, simple, value),
        _ => Err(anyhow!(
            "Field '{}' is not a simple value; complex structures are not supported yet",
            field
        )),
    }
}

fn set_simple_value_from_yaml(
    field: &str,
    simple: SimpleValueMut<'_>,
    value: &serde_yaml::Value,
) -> Result<()> {
    match simple {
        SimpleValueMut::Boolean(v) => {
            *v = value
                .as_bool()
                .ok_or_else(|| anyhow!("Field '{}' expects bool", field))?;
            Ok(())
        }
        SimpleValueMut::Int64(v) => {
            *v = value
                .as_i64()
                .ok_or_else(|| anyhow!("Field '{}' expects i64", field))?;
            Ok(())
        }
        SimpleValueMut::Uint64(v) => {
            *v = value
                .as_u64()
                .ok_or_else(|| anyhow!("Field '{}' expects u64", field))?;
            Ok(())
        }
        SimpleValueMut::Int32(v) => {
            let n = value
                .as_i64()
                .ok_or_else(|| anyhow!("Field '{}' expects i32", field))?;
            *v = i32::try_from(n).map_err(|_| anyhow!("Field '{}' i32 out of range", field))?;
            Ok(())
        }
        SimpleValueMut::Uint32(v) => {
            let n = value
                .as_u64()
                .ok_or_else(|| anyhow!("Field '{}' expects u32", field))?;
            *v = u32::try_from(n).map_err(|_| anyhow!("Field '{}' u32 out of range", field))?;
            Ok(())
        }
        SimpleValueMut::Int16(v) => {
            let n = value
                .as_i64()
                .ok_or_else(|| anyhow!("Field '{}' expects i16", field))?;
            *v = i16::try_from(n).map_err(|_| anyhow!("Field '{}' i16 out of range", field))?;
            Ok(())
        }
        SimpleValueMut::Uint16(v) => {
            let n = value
                .as_u64()
                .ok_or_else(|| anyhow!("Field '{}' expects u16", field))?;
            *v = u16::try_from(n).map_err(|_| anyhow!("Field '{}' u16 out of range", field))?;
            Ok(())
        }
        SimpleValueMut::Int8(v) => {
            let n = value
                .as_i64()
                .ok_or_else(|| anyhow!("Field '{}' expects i8", field))?;
            *v = i8::try_from(n).map_err(|_| anyhow!("Field '{}' i8 out of range", field))?;
            Ok(())
        }
        SimpleValueMut::Uint8(v) | SimpleValueMut::Octet(v) | SimpleValueMut::Char(v) => {
            let n = value
                .as_u64()
                .ok_or_else(|| anyhow!("Field '{}' expects u8", field))?;
            *v = u8::try_from(n).map_err(|_| anyhow!("Field '{}' u8 out of range", field))?;
            Ok(())
        }
        SimpleValueMut::Float(v) => {
            *v = value
                .as_f64()
                .ok_or_else(|| anyhow!("Field '{}' expects f32", field))? as f32;
            Ok(())
        }
        SimpleValueMut::Double(v) => {
            *v = value
                .as_f64()
                .ok_or_else(|| anyhow!("Field '{}' expects f64", field))?;
            Ok(())
        }
        SimpleValueMut::String(s) => {
            let text = value
                .as_str()
                .ok_or_else(|| anyhow!("Field '{}' expects string", field))?;
            *s = text.into();
            Ok(())
        }
        SimpleValueMut::Message(mut nested) => {
            let map = value
                .as_mapping()
                .ok_or_else(|| anyhow!("Field '{}' expects mapping for nested message", field))?;

            for (k, v) in map {
                let key = k
                    .as_str()
                    .ok_or_else(|| anyhow!("Nested field name must be a string"))?;
                let Some(nested_field) = nested.get_mut(key) else {
                    return Err(anyhow!(
                        "Unknown nested field '{}.{}'",
                        field,
                        key
                    ));
                };
                match nested_field {
                    ValueMut::Simple(simple) => set_simple_value_from_yaml(key, simple, v)?,
                    _ => {
                        return Err(anyhow!(
                            "Nested field '{}.{}' is not a simple value; complex structures are not supported yet",
                            field,
                            key
                        ));
                    }
                }
            }
            Ok(())
        }
        other => Err(anyhow!(
            "Field '{}' type {:?} is not supported yet",
            field,
            other
        )),
    }
}

fn build_message(message_type: &str, yaml: &str) -> Result<DynamicMessage> {
    let msg_type: MessageTypeName = message_type
        .try_into()
        .map_err(|e| anyhow!("Invalid message type '{}': {}", message_type, e))?;
    let mut msg = DynamicMessage::new(msg_type)?;

    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml)
        .map_err(|e| anyhow!("Failed to parse YAML values: {}", e))?;

    let map = yaml_value
        .as_mapping()
        .ok_or_else(|| anyhow!("YAML message must be a mapping/object"))?;

    for (k, v) in map {
        let field = k
            .as_str()
            .ok_or_else(|| anyhow!("Top-level YAML keys must be strings"))?;
        set_field_from_yaml(&mut msg, field, v)?;
    }

    Ok(msg)
}

async fn run_command(matches: ArgMatches, _common_args: CommonTopicArgs) -> Result<()> {
    let topic_name = matches
        .get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?
        .clone();
    let message_type = matches
        .get_one::<String>("message_type")
        .ok_or_else(|| anyhow!("Message type is required"))?
        .clone();
    let yaml_values = matches
        .get_many::<String>("values")
        .ok_or_else(|| anyhow!("Message values are required"))
        .map(|vals| vals.cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    let rate_hz = matches
        .get_one::<String>("rate")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0)
        .max(0.0);
    let print_every = matches
        .get_one::<String>("print")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1)
        .max(1);
    let once = matches.get_flag("once");
    let times = matches
        .get_one::<String>("times")
        .and_then(|s| s.parse::<u64>().ok());
    let wait_matching_subscriptions = matches
        .get_one::<String>("wait_matching_subscriptions")
        .and_then(|s| s.parse::<usize>().ok());
    let keep_alive_secs = matches
        .get_one::<String>("keep_alive")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.1);
    let node_name = matches
        .get_one::<String>("node_name")
        .cloned()
        .unwrap_or_else(|| "roc_topic_pub".to_string());

    let yaml = yaml_values.join(" ");

    let context = Context::default_from_env()?;
    let executor = context.create_basic_executor();
    let node = executor.create_node(node_name.as_str())?;

    if let Some(required) = wait_matching_subscriptions {
        let start = Instant::now();
        let timeout = Duration::from_secs(5);
        while start.elapsed() < timeout {
            let subs = node.count_subscriptions(&topic_name)? as usize;
            if subs >= required {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    let msg_type: MessageTypeName = message_type
        .as_str()
        .try_into()
        .map_err(|e| anyhow!("Invalid message type '{}': {}", message_type, e))?;
    let publisher = node.create_dynamic_publisher(msg_type, topic_name.as_str())?;

    let mut published = 0_u64;
    let period = if rate_hz > 0.0 {
        Duration::from_secs_f64(1.0 / rate_hz)
    } else {
        Duration::from_secs(0)
    };

    loop {
        let msg = build_message(&message_type, &yaml)?;
        publisher.publish(msg)?;
        published += 1;

        if published % print_every == 0 {
            println!("published #{} to {}", published, topic_name);
        }

        if once {
            break;
        }
        if let Some(limit) = times {
            if published >= limit {
                break;
            }
        }

        if period != Duration::from_secs(0) {
            tokio::time::sleep(period).await;
        } else {
            // rate 0 -> publish as fast as possible, yield so ctrl+c can interrupt.
            tokio::task::yield_now().await;
        }
    }

    if keep_alive_secs > 0.0 {
        tokio::time::sleep(Duration::from_secs_f64(keep_alive_secs)).await;
    }

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(run_command(matches, common_args)) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
