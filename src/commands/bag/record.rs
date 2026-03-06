use crate::commands::cli::handle_anyhow_result;
use anyhow::{Result, anyhow};
use clap::ArgMatches;
use colored::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use mcap::records::MessageHeader;
use mcap::{WriteOptions, Writer};

use crate::graph::RclGraphContext;
use crate::shared::serialized_transport::{SerializedReceiver, sleep_short};

fn now_nanos() -> u64 {
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    (dur.as_secs() * 1_000_000_000) + (dur.subsec_nanos() as u64)
}

fn resolve_topic_types(topics: &[String]) -> Result<BTreeMap<String, String>> {
    let ctx = RclGraphContext::new()?;
    let all = ctx.get_topic_names_and_types()?;
    let mut map: BTreeMap<String, String> = BTreeMap::new();

    for t in topics {
        for (name, ty) in &all {
            if name == t {
                map.insert(t.clone(), ty.clone());
                break;
            }
        }
        if !map.contains_key(t) {
            return Err(anyhow!("Could not resolve type for topic '{}'", t));
        }
    }

    Ok(map)
}

fn run_command(matches: ArgMatches) -> Result<()> {
    let output = matches
        .get_one::<String>("output")
        .cloned()
        .unwrap_or_else(|| "out.mcap".to_string());

    let all = matches.get_flag("all");
    let separated = matches.get_flag("separated");
    let topics: Vec<String> = matches
        .get_many::<String>("topics")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let type_override = matches.get_one::<String>("type").cloned();

    let (topic_types, topics_to_record) = if all {
        return Err(anyhow!("--all recording not implemented yet"));
    } else {
        if topics.is_empty() {
            return Err(anyhow!("Please provide at least one topic or use --all"));
        }
        if type_override.is_some() && topics.len() != 1 {
            return Err(anyhow!("--type is only supported with a single topic"));
        }
        if let Some(ty) = type_override {
            let mut m = BTreeMap::new();
            m.insert(topics[0].clone(), ty);
            (m, topics)
        } else {
            (resolve_topic_types(&topics)?, topics)
        }
    };

    if separated && topics_to_record.len() <= 1 {
        // No-op.
    }

    println!("{}", "Recording MCAP:".bright_yellow().bold());
    println!("  {} {}", "Output:".bright_yellow(), output.bright_cyan());
    println!("  {}", "Topics:".bright_yellow());
    for t in &topics_to_record {
        let ty = topic_types
            .get(t)
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        println!("    {} {}", t.bright_cyan(), format!("[{}]", ty).green());
    }
    println!();

    // If separated, create one writer per topic.
    let mut per_topic_writers: Option<BTreeMap<String, (Writer<File>, u16, u32)>> = None;
    let mut shared_writer: Option<(Writer<File>, BTreeMap<String, u16>, BTreeMap<String, u32>)> =
        None;

    if separated && topics_to_record.len() > 1 {
        let mut map: BTreeMap<String, (Writer<File>, u16, u32)> = BTreeMap::new();
        let base = PathBuf::from(&output);
        let base_is_dir = base.is_dir();
        for t in &topics_to_record {
            let ty = topic_types
                .get(t)
                .cloned()
                .ok_or_else(|| anyhow!("Missing type for topic '{}'", t))?;
            let filename = if base_is_dir {
                let safe = t.trim_start_matches('/').replace('/', "__");
                base.join(format!("{}.mcap", safe))
            } else {
                let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
                let safe = t.trim_start_matches('/').replace('/', "__");
                let parent = base.parent().unwrap_or_else(|| Path::new("."));
                parent.join(format!("{}__{}.mcap", stem, safe))
            };

            let file = File::create(&filename)
                .map_err(|e| anyhow!("Failed to create {}: {}", filename.display(), e))?;
            let mut writer = Writer::with_options(file, WriteOptions::default())?;
            let schema_id = writer.add_schema(&ty, "ros2msg", &[])?;
            let channel_id = writer.add_channel(schema_id, t, "cdr", &BTreeMap::new())?;
            map.insert(t.clone(), (writer, channel_id, 0));
        }
        per_topic_writers = Some(map);
    } else {
        let file =
            File::create(&output).map_err(|e| anyhow!("Failed to create {}: {}", output, e))?;
        let mut writer = Writer::with_options(file, WriteOptions::default())?;

        let mut schema_ids: BTreeMap<String, u16> = BTreeMap::new();
        let mut channel_ids: BTreeMap<String, u16> = BTreeMap::new();
        let mut seq: BTreeMap<String, u32> = BTreeMap::new();

        for t in &topics_to_record {
            let ty = topic_types
                .get(t)
                .ok_or_else(|| anyhow!("Missing type for topic '{}'", t))?
                .clone();
            let sid = if let Some(id) = schema_ids.get(&ty) {
                *id
            } else {
                let id = writer.add_schema(&ty, "ros2msg", &[])?;
                schema_ids.insert(ty.clone(), id);
                id
            };
            let cid = writer.add_channel(sid, t, "cdr", &BTreeMap::new())?;
            channel_ids.insert(t.clone(), cid);
            seq.insert(t.clone(), 0);
        }

        shared_writer = Some((writer, channel_ids, seq));
    }

    let mut receivers: Vec<(String, String, SerializedReceiver)> = Vec::new();
    for t in &topics_to_record {
        let ty = topic_types
            .get(t)
            .cloned()
            .ok_or_else(|| anyhow!("Missing type for topic '{}'", t))?;
        receivers.push((t.clone(), ty.clone(), SerializedReceiver::new(t, &ty)?));
    }

    let mut total: u64 = 0;

    loop {
        sleep_short();
        let mut wrote_any = false;
        for (topic, _ty, rx) in receivers.iter_mut() {
            if let Some(bytes) = rx.take()? {
                let t = now_nanos();

                if let Some(map) = per_topic_writers.as_mut() {
                    let (writer, channel_id, seq) = map
                        .get_mut(topic)
                        .ok_or_else(|| anyhow!("Missing writer for topic '{}'", topic))?;
                    writer.write_to_known_channel(
                        &MessageHeader {
                            channel_id: *channel_id,
                            sequence: *seq,
                            log_time: t,
                            publish_time: t,
                        },
                        &bytes,
                    )?;
                    *seq = seq.wrapping_add(1);
                } else if let Some((writer, channel_ids, seqs)) = shared_writer.as_mut() {
                    let cid = *channel_ids
                        .get(topic)
                        .ok_or_else(|| anyhow!("Missing channel id for topic '{}'", topic))?;
                    let s = seqs
                        .get_mut(topic)
                        .ok_or_else(|| anyhow!("Missing sequence counter for topic '{}'", topic))?;
                    writer.write_to_known_channel(
                        &MessageHeader {
                            channel_id: cid,
                            sequence: *s,
                            log_time: t,
                            publish_time: t,
                        },
                        &bytes,
                    )?;
                    *s = s.wrapping_add(1);
                }

                total += 1;
                wrote_any = true;
            }
        }

        if wrote_any && total % 50 == 0 {
            println!(
                "{} {}",
                "Recorded".bright_green(),
                total.to_string().bright_white()
            );
        }
    }
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
