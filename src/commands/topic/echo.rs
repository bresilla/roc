use crate::arguments::topic::CommonTopicArgs;
use crate::commands::cli::run_async_command;
use crate::graph::RclGraphContext;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use serde_yaml::{Mapping, Value as YamlValue};

use rclrs::{
    ArrayValue, BoundedSequenceValue, DynamicMessageView, SequenceValue, SimpleValue, Value,
};

#[derive(Debug, Clone)]
struct EchoOptions {
    topic_name: String,
    field: Option<String>,
    full_length: bool,
    truncate_length: usize,
    no_arr: bool,
    no_str: bool,
    flow_style: bool,
    no_lost_messages: bool,
    raw: bool,
    once: bool,
    csv: bool,
}

impl EchoOptions {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        let topic_name = matches
            .get_one::<String>("topic_name")
            .ok_or_else(|| anyhow!("Topic name is required"))?
            .clone();

        let field = matches.get_one::<String>("field").cloned();
        let full_length = matches.get_flag("full_length");
        let truncate_length = matches
            .get_one::<String>("truncate_length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(128);
        let no_arr = matches.get_flag("no_arr");
        let no_str = matches.get_flag("no_str");
        let flow_style = matches.get_flag("flow_style");
        let no_lost_messages = matches.get_flag("no_lost_messages");
        let raw = matches.get_flag("raw");
        let once = matches.get_flag("once");
        let csv = matches.get_flag("csv");

        Ok(EchoOptions {
            topic_name,
            field,
            full_length,
            truncate_length,
            no_arr,
            no_str,
            flow_style,
            no_lost_messages,
            raw,
            once,
            csv,
        })
    }
}

fn echo_capability_note() -> &'static str {
    "Native echo supports formatted dynamic-message output. Raw serialized output is not available on this path."
}

fn truncate_string_if_needed(s: &str, opts: &EchoOptions) -> String {
    if opts.full_length {
        return s.to_string();
    }
    let max = opts.truncate_length;
    if s.chars().count() <= max {
        return s.to_string();
    }
    let prefix: String = s.chars().take(max).collect();
    format!("{}...", prefix)
}

fn maybe_truncate_sequence(mut seq: Vec<YamlValue>, opts: &EchoOptions) -> Vec<YamlValue> {
    if opts.full_length {
        return seq;
    }
    let max = opts.truncate_length;
    if seq.len() <= max {
        return seq;
    }
    seq.truncate(max);
    seq.push(YamlValue::String("...".to_string()));
    seq
}

fn simple_to_yaml(v: &SimpleValue<'_>, opts: &EchoOptions) -> Result<YamlValue> {
    Ok(match v {
        SimpleValue::Float(x) => YamlValue::from(**x),
        SimpleValue::Double(x) => YamlValue::from(**x),
        SimpleValue::LongDouble(_ptr) => YamlValue::String("<long double>".to_string()),
        SimpleValue::Char(x) => YamlValue::from(**x),
        SimpleValue::WChar(x) => YamlValue::from(**x),
        SimpleValue::Boolean(b) => YamlValue::from(**b),
        SimpleValue::Octet(x) => YamlValue::from(**x),
        SimpleValue::Uint8(x) => YamlValue::from(**x),
        SimpleValue::Int8(x) => YamlValue::from(**x),
        SimpleValue::Uint16(x) => YamlValue::from(**x),
        SimpleValue::Int16(x) => YamlValue::from(**x),
        SimpleValue::Uint32(x) => YamlValue::from(**x),
        SimpleValue::Int32(x) => YamlValue::from(**x),
        SimpleValue::Uint64(x) => YamlValue::from(**x),
        SimpleValue::Int64(x) => YamlValue::from(**x),
        SimpleValue::String(s) => {
            if opts.no_str {
                YamlValue::String("<string suppressed>".to_string())
            } else {
                YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
            }
        }
        SimpleValue::BoundedString(s) => {
            if opts.no_str {
                YamlValue::String("<string suppressed>".to_string())
            } else {
                YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
            }
        }
        SimpleValue::WString(s) => {
            if opts.no_str {
                YamlValue::String("<wstring suppressed>".to_string())
            } else {
                YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
            }
        }
        SimpleValue::BoundedWString(s) => {
            if opts.no_str {
                YamlValue::String("<wstring suppressed>".to_string())
            } else {
                YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
            }
        }
        SimpleValue::Message(m) => message_view_to_yaml(m, opts)?,
    })
}

fn array_to_yaml(v: &ArrayValue<'_>, opts: &EchoOptions) -> Result<YamlValue> {
    if opts.no_arr {
        return Ok(YamlValue::String("<array suppressed>".to_string()));
    }

    let seq: Vec<YamlValue> = match v {
        ArrayValue::FloatArray(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::DoubleArray(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::LongDoubleArray(_ptr, len) => {
            vec![YamlValue::String(format!("<long double[{len}]>"))]
        }
        ArrayValue::CharArray(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::WCharArray(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::BooleanArray(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::OctetArray(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Uint8Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Int8Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Uint16Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Int16Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Uint32Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Int32Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Uint64Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::Int64Array(a) => a.iter().map(|x| YamlValue::from(*x)).collect(),
        ArrayValue::StringArray(a) => {
            if opts.no_str {
                vec![YamlValue::String("<string array suppressed>".to_string())]
            } else {
                a.iter()
                    .map(|s| {
                        YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        ArrayValue::BoundedStringArray(a) => {
            if opts.no_str {
                vec![YamlValue::String("<string array suppressed>".to_string())]
            } else {
                a.iter()
                    .map(|s| {
                        YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        ArrayValue::WStringArray(a) => {
            if opts.no_str {
                vec![YamlValue::String("<wstring array suppressed>".to_string())]
            } else {
                a.iter()
                    .map(|s| {
                        YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        ArrayValue::BoundedWStringArray(a) => {
            if opts.no_str {
                vec![YamlValue::String("<wstring array suppressed>".to_string())]
            } else {
                a.iter()
                    .map(|s| {
                        YamlValue::String(truncate_string_if_needed(s.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        ArrayValue::MessageArray(a) => a
            .iter()
            .map(|m| message_view_to_yaml(m, opts))
            .collect::<Result<Vec<_>>>()?,
    };

    Ok(YamlValue::Sequence(maybe_truncate_sequence(seq, opts)))
}

fn sequence_to_yaml(v: &SequenceValue<'_>, opts: &EchoOptions) -> Result<YamlValue> {
    if opts.no_arr {
        return Ok(YamlValue::String("<sequence suppressed>".to_string()));
    }

    let seq: Vec<YamlValue> = match v {
        SequenceValue::FloatSequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::DoubleSequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::LongDoubleSequence(_ptr) => {
            vec![YamlValue::String("<long double sequence>".to_string())]
        }
        SequenceValue::CharSequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::WCharSequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::BooleanSequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::OctetSequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Uint8Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Int8Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Uint16Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Int16Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Uint32Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Int32Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Uint64Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::Int64Sequence(s) => s.iter().map(|x| YamlValue::from(*x)).collect(),
        SequenceValue::StringSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<string sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        SequenceValue::BoundedStringSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<string sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        SequenceValue::WStringSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<wstring sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        SequenceValue::BoundedWStringSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<wstring sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        SequenceValue::MessageSequence(s) => s
            .iter()
            .map(|m| message_view_to_yaml(m, opts))
            .collect::<Result<Vec<_>>>()?,
    };

    Ok(YamlValue::Sequence(maybe_truncate_sequence(seq, opts)))
}

fn bounded_sequence_to_yaml(v: &BoundedSequenceValue<'_>, opts: &EchoOptions) -> Result<YamlValue> {
    // Treat the same as unbounded sequence for printing.
    if opts.no_arr {
        return Ok(YamlValue::String(
            "<bounded sequence suppressed>".to_string(),
        ));
    }
    let seq: Vec<YamlValue> = match v {
        BoundedSequenceValue::FloatBoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::DoubleBoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::LongDoubleBoundedSequence(_ptr, ub) => {
            vec![YamlValue::String(format!(
                "<long double bounded sequence (max {ub})>"
            ))]
        }
        BoundedSequenceValue::CharBoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::WCharBoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::BooleanBoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::OctetBoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Uint8BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Int8BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Uint16BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Int16BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Uint32BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Int32BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Uint64BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::Int64BoundedSequence(s) => {
            s.iter().map(|x| YamlValue::from(*x)).collect()
        }
        BoundedSequenceValue::StringBoundedSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<string sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        BoundedSequenceValue::BoundedStringBoundedSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<string sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        BoundedSequenceValue::WStringBoundedSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<wstring sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        BoundedSequenceValue::BoundedWStringBoundedSequence(s) => {
            if opts.no_str {
                vec![YamlValue::String(
                    "<wstring sequence suppressed>".to_string(),
                )]
            } else {
                s.iter()
                    .map(|st| {
                        YamlValue::String(truncate_string_if_needed(st.to_string().as_str(), opts))
                    })
                    .collect()
            }
        }
        BoundedSequenceValue::MessageBoundedSequence(s) => s
            .iter()
            .map(|m| message_view_to_yaml(m, opts))
            .collect::<Result<Vec<_>>>()?,
    };

    Ok(YamlValue::Sequence(maybe_truncate_sequence(seq, opts)))
}

fn value_to_yaml(v: &Value<'_>, opts: &EchoOptions) -> Result<YamlValue> {
    match v {
        Value::Simple(s) => simple_to_yaml(s, opts),
        Value::Array(a) => array_to_yaml(a, opts),
        Value::Sequence(s) => sequence_to_yaml(s, opts),
        Value::BoundedSequence(s) => bounded_sequence_to_yaml(s, opts),
    }
}

fn message_view_to_yaml(view: &DynamicMessageView<'_>, opts: &EchoOptions) -> Result<YamlValue> {
    let mut map = Mapping::new();
    for (name, v) in view.iter() {
        map.insert(
            YamlValue::String(name.to_string()),
            value_to_yaml(&v, opts)?,
        );
    }
    Ok(YamlValue::Mapping(map))
}

fn parse_field_selector(selector: &str) -> Vec<FieldSelectorPart> {
    // Supports paths like: "pose.position.x" and indexes like "poses[0].position".
    // This is intentionally minimal.
    let mut out = Vec::new();
    for part in selector.split('.') {
        let mut rest = part;
        loop {
            if let Some(idx_start) = rest.find('[') {
                let name = &rest[..idx_start];
                if !name.is_empty() {
                    out.push(FieldSelectorPart::Field(name.to_string()));
                }
                if let Some(idx_end) = rest[idx_start..].find(']') {
                    let inside = &rest[idx_start + 1..idx_start + idx_end];
                    if let Ok(i) = inside.parse::<usize>() {
                        out.push(FieldSelectorPart::Index(i));
                    }
                    rest = &rest[idx_start + idx_end + 1..];
                    if rest.is_empty() {
                        break;
                    }
                } else {
                    // Malformed, treat as a field and stop.
                    out.push(FieldSelectorPart::Field(rest.to_string()));
                    break;
                }
            } else {
                out.push(FieldSelectorPart::Field(rest.to_string()));
                break;
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
enum FieldSelectorPart {
    Field(String),
    Index(usize),
}

fn select_field<'a>(mut current: Value<'a>, selector: &[FieldSelectorPart]) -> Option<Value<'a>> {
    for part in selector {
        match part {
            FieldSelectorPart::Field(name) => {
                let Value::Simple(SimpleValue::Message(m)) = current else {
                    return None;
                };
                current = m.get(name)?;
            }
            FieldSelectorPart::Index(i) => {
                // rclrs dynamic "container" value types do not currently allow building a
                // new `Value<'a>` borrowing from the same message view in a sound way, because
                // indexing returns references tied to a short-lived borrow of the container.
                //
                // Keep the parser accepting "[idx]" for future compatibility, but treat it as
                // unsupported at runtime for now.
                let _ = i;
                return None;
            }
        }
    }
    Some(current)
}

fn format_value_for_csv(v: &YamlValue) -> String {
    match v {
        YamlValue::Null => "".to_string(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Number(n) => n.to_string(),
        YamlValue::String(s) => s.replace('"', "\"\""),
        _ => {
            // Fallback: dump YAML on one line.
            let s = serde_yaml::to_string(v).unwrap_or_else(|_| "".to_string());
            s.replace('\n', " ").trim().to_string().replace('"', "\"\"")
        }
    }
}

async fn echo_topic_messages(
    options: EchoOptions,
    _common_args: CommonTopicArgs,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Create RCL context for subscription
    let graph_context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Wait for topic to appear
    if !graph_context.wait_for_topic(&options.topic_name, Duration::from_secs(3))? {
        return Err(anyhow!(
            "Topic '{}' not found after waiting",
            options.topic_name
        ));
    }

    // Get topic type
    let topic_type = {
        let topics_and_types = graph_context
            .get_topic_names_and_types()
            .map_err(|e| anyhow!("Failed to get topic types: {}", e))?;

        topics_and_types
            .iter()
            .find(|(name, _)| name == &options.topic_name)
            .map(|(_, type_name)| type_name.clone())
            .ok_or_else(|| {
                anyhow!(
                    "Could not determine type for topic '{}'",
                    options.topic_name
                )
            })?
    };

    if options.flow_style {
        eprintln!("Note: --flow-style is not fully supported yet; using default YAML formatting");
    }
    if options.raw {
        eprintln!("Note: {}", echo_capability_note());
    }

    // Wait for publishers to be available
    if !graph_context.wait_for_topic_with_publishers(&options.topic_name, Duration::from_secs(5))? {
        if !options.no_lost_messages {
            eprintln!("WARNING: no publisher on [{}]", options.topic_name);
        }
    }

    // Create dynamic subscription using our new infrastructure
    let subscription = graph_context.create_subscription(&options.topic_name, &topic_type)?;

    println!(
        "Subscribed to [{}] (type: {})",
        options.topic_name, topic_type
    );

    let mut message_count = 0;
    let check_interval = Duration::from_millis(50); // 20 Hz polling

    let selector = options.field.as_deref().map(parse_field_selector);
    let mut last_no_publisher_warning: Option<std::time::Instant> = None;

    // Main message reception loop
    while running.load(Ordering::Relaxed) {
        sleep(check_interval).await;

        // Check for new messages
        match subscription.take_message() {
            Ok(Some(received)) => {
                message_count += 1;

                let view = received.message.view();

                let output = if let Some(ref selector) = selector {
                    let Some(v) = view.get(
                        &selector
                            .iter()
                            .find_map(|p| {
                                if let FieldSelectorPart::Field(name) = p {
                                    Some(name.as_str())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(""),
                    ) else {
                        return Err(anyhow!(
                            "Field '{}' not found",
                            options.field.clone().unwrap_or_default()
                        ));
                    };

                    // If the selector starts with an index, or has deeper parts, do a full traversal.
                    let selected = if selector.len() == 1 {
                        Some(v)
                    } else {
                        select_field(v, &selector[1..])
                    };
                    let Some(selected) = selected else {
                        return Err(anyhow!(
                            "Field '{}' not found",
                            options.field.clone().unwrap_or_default()
                        ));
                    };
                    let yaml_v = value_to_yaml(&selected, &options)?;
                    if options.csv {
                        format_value_for_csv(&yaml_v)
                    } else {
                        serde_yaml::to_string(&yaml_v)?.trim_end().to_string()
                    }
                } else {
                    let yaml_v = message_view_to_yaml(&view, &options)?;
                    if options.csv {
                        format_value_for_csv(&yaml_v)
                    } else {
                        serde_yaml::to_string(&yaml_v)?.trim_end().to_string()
                    }
                };

                // Format output based on options
                if options.csv {
                    // CSV format with header only on first message
                    if message_count == 1 {
                        println!("timestamp,seq,data");
                    }
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|error| anyhow!("System clock is before UNIX epoch: {}", error))?;
                    println!(
                        "{},{},\"{}\"",
                        current_time.as_secs(),
                        message_count,
                        output.replace('"', "\"\"") // Escape quotes for CSV
                    );
                } else {
                    // YAML format (default)
                    println!("{}", output);
                    if !options.csv {
                        println!("---");
                    }
                }

                if options.once {
                    break;
                }
            }
            Ok(None) => {
                // No message available, continue polling
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }

        // Check if publishers are still active
        let current_publisher_count = graph_context
            .count_publishers(&options.topic_name)
            .unwrap_or(0);
        if current_publisher_count == 0 && !options.no_lost_messages {
            // Only show this message if we haven't received any messages recently
            let now = std::time::Instant::now();
            let should_warn = match last_no_publisher_warning {
                Some(last_time) => now.duration_since(last_time) > Duration::from_secs(5),
                None => true,
            };

            if should_warn {
                eprintln!("WARNING: no publisher on [{}]", options.topic_name);
                last_no_publisher_warning = Some(now);
            }
        }
    }

    Ok(())
}

// NOTE: Formatting here is "ros2-cli-like" but not a byte-for-byte match.
// We intentionally keep it fully native (no `ros2 topic echo` subprocess).

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let options = EchoOptions::from_matches(&matches)?;

    // Handle common arguments silently (like ros2 topic echo does)
    // Only show QoS notes if explicitly set
    if let Some(qos_profile) = matches.get_one::<String>("qos_profile") {
        eprintln!("Note: Using QoS profile: {}", qos_profile);
    }

    if let Some(qos_depth) = matches.get_one::<String>("qos_depth") {
        eprintln!("Note: Using QoS depth: {}", qos_depth);
    }

    if let Some(qos_history) = matches.get_one::<String>("qos_history") {
        eprintln!("Note: Using QoS history: {}", qos_history);
    }

    if let Some(qos_reliability) = matches.get_one::<String>("qos_reliability") {
        eprintln!("Note: Using QoS reliability: {}", qos_reliability);
    }

    if let Some(qos_durability) = matches.get_one::<String>("qos_durability") {
        eprintln!("Note: Using QoS durability: {}", qos_durability);
    }

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to listen for ctrl+c: {}", error);
            return;
        }
        running_clone.store(false, Ordering::Relaxed);
    });

    // Start echoing messages
    echo_topic_messages(options, common_args, running).await
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    run_async_command(run_command(matches, common_args));
}

#[cfg(test)]
mod tests {
    use super::echo_capability_note;

    #[test]
    fn echo_capability_note_mentions_raw_output_limit() {
        assert!(echo_capability_note().contains("Raw serialized output"));
    }
}
