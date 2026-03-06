use crate::commands::cli::handle_anyhow_result;
use anyhow::{Result, anyhow};
use clap::ArgMatches;
use colored::*;
use std::collections::BTreeMap;
use std::fs;
use std::time::{Duration, Instant};

use memmap2::Mmap;

use crate::shared::serialized_transport::{SerializedSender, sleep_short};

#[derive(Debug, Clone)]
struct McapMessage {
    topic: String,
    msg_type: String,
    log_time: u64,
    publish_time: u64,
    data: Vec<u8>,
}

fn map_file(path: &str) -> Result<Mmap> {
    let file = fs::File::open(path).map_err(|e| anyhow!("Failed to open {}: {}", path, e))?;
    unsafe { Mmap::map(&file).map_err(|e| anyhow!("Failed to mmap {}: {}", path, e)) }
}

fn read_messages(path: &str) -> Result<Vec<McapMessage>> {
    let mapped = map_file(path)?;
    let mut out = Vec::new();

    // We collect schemas and channels to recover message type.
    let mut schemas: BTreeMap<u16, String> = BTreeMap::new();
    let mut channels: BTreeMap<u16, (String, u16)> = BTreeMap::new(); // channel_id -> (topic, schema_id)

    for rec in mcap::read::LinearReader::new(&mapped)?.into_iter() {
        let rec = rec?;
        match rec {
            mcap::records::Record::Schema { header, .. } => {
                schemas.insert(header.id, header.name);
            }
            mcap::records::Record::Channel(c) => {
                channels.insert(c.id, (c.topic, c.schema_id));
            }
            mcap::records::Record::Message { header, data } => {
                let Some((topic, schema_id)) = channels.get(&header.channel_id).cloned() else {
                    continue;
                };
                let msg_type = schemas
                    .get(&schema_id)
                    .cloned()
                    .unwrap_or_else(|| "<unknown>".to_string());
                out.push(McapMessage {
                    topic,
                    msg_type,
                    log_time: header.log_time,
                    publish_time: header.publish_time,
                    data: data.into_owned(),
                });
            }
            _ => {}
        }
    }

    Ok(out)
}

fn run_command(matches: ArgMatches) -> Result<()> {
    let paths: Vec<String> = matches
        .get_many::<String>("PATHS")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    if paths.is_empty() {
        return Err(anyhow!("At least one MCAP path is required"));
    }

    let rate = matches
        .get_one::<String>("rate")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0)
        .max(0.0001);
    let loop_play = matches.get_flag("loop");

    let mut all_messages: Vec<McapMessage> = Vec::new();
    for p in &paths {
        let mut msgs = read_messages(p)?;
        all_messages.append(&mut msgs);
    }
    if all_messages.is_empty() {
        return Err(anyhow!("No messages found in input MCAP files"));
    }

    all_messages.sort_by(|a, b| a.log_time.cmp(&b.log_time));

    println!("{}", "Playing MCAP:".bright_yellow().bold());
    for p in &paths {
        println!("  {}", p.bright_cyan());
    }
    println!(
        "  {} {}",
        "Rate:".bright_yellow(),
        rate.to_string().bright_white()
    );
    println!();

    // Create publishers lazily per (topic, type).
    let mut pubs: BTreeMap<(String, String), SerializedSender> = BTreeMap::new();

    loop {
        let t0 = all_messages[0].log_time;
        let start = Instant::now();

        for m in &all_messages {
            let dt_ns = m.log_time.saturating_sub(t0);
            let dt = Duration::from_nanos(((dt_ns as f64) / rate) as u64);
            while start.elapsed() < dt {
                sleep_short();
            }

            let key = (m.topic.clone(), m.msg_type.clone());
            if !pubs.contains_key(&key) {
                let pub_ = SerializedSender::new(&m.topic, &m.msg_type)?;
                pubs.insert(key.clone(), pub_);
            }
            let publisher = pubs.get_mut(&key).ok_or_else(|| {
                anyhow!("Missing publisher for topic '{}' [{}]", m.topic, m.msg_type)
            })?;
            publisher.publish(&m.data)?;
        }

        if !loop_play {
            break;
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
