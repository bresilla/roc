use crate::arguments::topic::CommonTopicArgs;
use crate::commands::cli::{install_ctrlc_flag, print_error_and_exit};
use crate::ui::{
    blocks,
    output::{self, OutputMode},
};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use rclrs::{
    Context, CreateBasicExecutor, DynamicMessage, MessageTypeName, SimpleValueMut, ValueMut,
};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn set_field_from_yaml(
    msg: &mut DynamicMessage,
    field: &str,
    value: &serde_yaml::Value,
) -> Result<()> {
    let Some(field_value) = msg.get_mut(field) else {
        return Err(anyhow!(
            "Unknown field '{}' for type {}",
            field,
            msg.structure().type_name
        ));
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
                    return Err(anyhow!("Unknown nested field '{}.{}'", field, key));
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

    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| anyhow!("Failed to parse YAML values: {}", e))?;

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

fn publish_mode_label(once: bool, times: Option<u64>) -> String {
    if once {
        "once".to_string()
    } else if let Some(limit) = times {
        format!("{limit} messages")
    } else {
        "continuous".to_string()
    }
}

fn print_publish_header(
    output_mode: OutputMode,
    topic_name: &str,
    message_type: &str,
    node_name: &str,
    rate_hz: f64,
    print_every: u64,
    wait_matching_subscriptions: Option<usize>,
    keep_alive_secs: f64,
    mode_label: &str,
) {
    match output_mode {
        OutputMode::Human => {
            blocks::print_section("Topic Publish");
            blocks::print_field("Topic", topic_name);
            blocks::print_field("Type", message_type);
            blocks::print_field("Node", node_name);
            blocks::print_field("Rate", format!("{rate_hz:.3} Hz"));
            blocks::print_field("Print Every", print_every);
            blocks::print_field("Mode", mode_label);
            if let Some(required) = wait_matching_subscriptions {
                blocks::print_field("Wait Subs", required);
            }
            blocks::print_field("Keep Alive", format!("{keep_alive_secs:.3}s"));
            println!();
            if mode_label == "continuous" {
                blocks::print_note("Press Ctrl+C to stop");
            }
        }
        OutputMode::Plain => {
            output::print_plain_section("topic-publish");
            output::print_plain_field("topic", topic_name);
            output::print_plain_field("type", message_type);
            output::print_plain_field("node", node_name);
            output::print_plain_field("rate_hz", format!("{rate_hz:.3}"));
            output::print_plain_field("print_every", print_every);
            output::print_plain_field("mode", mode_label);
            if let Some(required) = wait_matching_subscriptions {
                output::print_plain_field("wait_matching_subscriptions", required);
            }
            output::print_plain_field("keep_alive_secs", format!("{keep_alive_secs:.3}"));
        }
        OutputMode::Json => {}
    }
}

fn print_wait_status(output_mode: OutputMode, topic_name: &str, required: usize) {
    match output_mode {
        OutputMode::Human => blocks::print_status(
            "WAIT",
            &[
                ("topic", topic_name.to_string()),
                ("subscriptions", required.to_string()),
            ],
        ),
        OutputMode::Plain => output::print_plain_status(
            "wait",
            &[
                ("topic", topic_name.to_string()),
                ("subscriptions", required.to_string()),
            ],
        ),
        OutputMode::Json => {}
    }
}

fn print_publish_progress(
    output_mode: OutputMode,
    published: u64,
    topic_name: &str,
    elapsed_secs: f64,
) {
    match output_mode {
        OutputMode::Human => blocks::print_status(
            "PUB",
            &[
                ("count", published.to_string()),
                ("topic", topic_name.to_string()),
                ("elapsed", format!("{elapsed_secs:.2}s")),
            ],
        ),
        OutputMode::Plain => output::print_plain_status(
            "pub",
            &[
                ("count", published.to_string()),
                ("topic", topic_name.to_string()),
                ("elapsed_secs", format!("{elapsed_secs:.2}")),
            ],
        ),
        OutputMode::Json => {}
    }
}

fn print_publish_summary(
    output_mode: OutputMode,
    topic_name: &str,
    message_type: &str,
    node_name: &str,
    rate_hz: f64,
    print_every: u64,
    mode_label: &str,
    wait_matching_subscriptions: Option<usize>,
    keep_alive_secs: f64,
    published: u64,
    elapsed_secs: f64,
    interrupted: bool,
) -> Result<()> {
    match output_mode {
        OutputMode::Human => {
            println!();
            blocks::print_section("Publish Summary");
            blocks::print_field("Topic", topic_name);
            blocks::print_field("Messages", published);
            blocks::print_field("Elapsed", format!("{elapsed_secs:.2}s"));
            blocks::print_success("Publishing stopped");
        }
        OutputMode::Plain => {
            output::print_plain_section("publish-summary");
            output::print_plain_field("topic", topic_name);
            output::print_plain_field("type", message_type);
            output::print_plain_field("node", node_name);
            output::print_plain_field("rate_hz", format!("{rate_hz:.3}"));
            output::print_plain_field("print_every", print_every);
            output::print_plain_field("mode", mode_label);
            if let Some(required) = wait_matching_subscriptions {
                output::print_plain_field("wait_matching_subscriptions", required);
            }
            output::print_plain_field("keep_alive_secs", format!("{keep_alive_secs:.3}"));
            output::print_plain_field("messages", published);
            output::print_plain_field("elapsed_secs", format!("{elapsed_secs:.3}"));
            output::print_plain_field("interrupted", interrupted);
            output::print_plain_field("status", "ok");
        }
        OutputMode::Json => {
            output::print_json(&json!({
                "command": "topic pub",
                "topic": topic_name,
                "type": message_type,
                "node": node_name,
                "rate_hz": rate_hz,
                "print_every": print_every,
                "mode": mode_label,
                "wait_matching_subscriptions": wait_matching_subscriptions,
                "keep_alive_secs": keep_alive_secs,
                "messages": published,
                "elapsed_secs": elapsed_secs,
                "interrupted": interrupted,
                "status": "ok"
            }))?;
        }
    }
    Ok(())
}

fn print_publish_error(
    output_mode: OutputMode,
    topic_name: Option<&str>,
    message_type: Option<&str>,
    error: &str,
) {
    match output_mode {
        OutputMode::Human => print_error_and_exit(error),
        OutputMode::Plain => {
            output::print_plain_section("topic-publish-error");
            if let Some(topic_name) = topic_name {
                output::print_plain_field("topic", topic_name);
            }
            if let Some(message_type) = message_type {
                output::print_plain_field("type", message_type);
            }
            output::print_plain_field("status", "error");
            output::print_plain_field("error", error);
            std::process::exit(1);
        }
        OutputMode::Json => {
            let _ = output::print_json(&json!({
                "command": "topic pub",
                "topic": topic_name,
                "type": message_type,
                "status": "error",
                "error": error
            }));
            std::process::exit(1);
        }
    }
}

async fn run_command(matches: ArgMatches, _common_args: CommonTopicArgs) -> Result<()> {
    let output_mode = OutputMode::from_matches(&matches);
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
    let mode_label = publish_mode_label(once, times);

    print_publish_header(
        output_mode,
        &topic_name,
        &message_type,
        &node_name,
        rate_hz,
        print_every,
        wait_matching_subscriptions,
        keep_alive_secs,
        &mode_label,
    );

    let context = Context::default_from_env()?;
    let executor = context.create_basic_executor();
    let node = executor.create_node(node_name.as_str())?;

    if let Some(required) = wait_matching_subscriptions {
        print_wait_status(output_mode, &topic_name, required);
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
    let running = Arc::new(AtomicBool::new(true));
    install_ctrlc_flag(Arc::clone(&running))?;
    let session_start = Instant::now();
    let mut interrupted = false;

    while running.load(Ordering::Relaxed) {
        let msg = build_message(&message_type, &yaml)?;
        publisher.publish(msg)?;
        published += 1;

        if published % print_every == 0 {
            print_publish_progress(
                output_mode,
                published,
                &topic_name,
                session_start.elapsed().as_secs_f64(),
            );
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
            let sleep_start = Instant::now();
            while running.load(Ordering::Relaxed) && sleep_start.elapsed() < period {
                let remaining = period.saturating_sub(sleep_start.elapsed());
                let chunk = remaining.min(Duration::from_millis(50));
                tokio::time::sleep(chunk).await;
            }
        } else {
            // rate 0 -> publish as fast as possible, yield so ctrl+c can interrupt.
            tokio::task::yield_now().await;
        }
    }

    if !once && times.map(|limit| published < limit).unwrap_or(true) && !running.load(Ordering::Relaxed) {
        interrupted = true;
    }

    if keep_alive_secs > 0.0 {
        tokio::time::sleep(Duration::from_secs_f64(keep_alive_secs)).await;
    }

    print_publish_summary(
        output_mode,
        &topic_name,
        &message_type,
        &node_name,
        rate_hz,
        print_every,
        &mode_label,
        wait_matching_subscriptions,
        keep_alive_secs,
        published,
        session_start.elapsed().as_secs_f64(),
        interrupted,
    )?;

    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    let output_mode = OutputMode::from_matches(&matches);
    let topic_name = matches.get_one::<String>("topic_name").cloned();
    let message_type = matches.get_one::<String>("message_type").cloned();
    let runtime = tokio::runtime::Runtime::new()
        .unwrap_or_else(|error| print_error_and_exit(format!("Failed to create async runtime: {error}")));

    if let Err(error) = runtime.block_on(run_command(matches, common_args)) {
        print_publish_error(
            output_mode,
            topic_name.as_deref(),
            message_type.as_deref(),
            &error.to_string(),
        );
    }
}
