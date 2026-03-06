use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Rosbag2Metadata {
    pub rosbag2_bagfile_information: Rosbag2BagfileInformation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rosbag2BagfileInformation {
    pub version: u32,
    pub storage_identifier: String,
    pub duration: Rosbag2Duration,
    #[allow(dead_code)]
    pub starting_time: Rosbag2Time,

    pub message_count: u64,
    pub topics_with_message_count: Vec<Rosbag2TopicWithMessageCount>,

    #[serde(default)]
    pub compression_format: String,
    #[serde(default)]
    pub compression_mode: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rosbag2Duration {
    pub nanoseconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rosbag2Time {
    #[allow(dead_code)]
    pub nanoseconds_since_epoch: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rosbag2TopicWithMessageCount {
    pub topic_metadata: Rosbag2TopicMetadata,
    pub message_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rosbag2TopicMetadata {
    pub name: String,
    pub r#type: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub serialization_format: String,
}

pub fn load_metadata(bag_dir: &Path) -> Result<Rosbag2Metadata> {
    let meta_path = bag_dir.join("metadata.yaml");
    let content = fs::read_to_string(&meta_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", meta_path.display(), e))?;
    let meta: Rosbag2Metadata =
        serde_yaml::from_str(&content).map_err(|e| anyhow!("Invalid metadata.yaml: {}", e))?;
    Ok(meta)
}

pub fn is_rosbag2_directory(path: &Path) -> bool {
    path.is_dir() && path.join("metadata.yaml").is_file()
}

pub fn find_rosbag2_directories(root: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Err(anyhow!("Path does not exist: {}", root.display()));
    }

    if is_rosbag2_directory(root) {
        out.push(root.to_path_buf());
        return Ok(out);
    }

    if !root.is_dir() {
        return Ok(out);
    }

    if recursive {
        for entry in walkdir::WalkDir::new(root)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if is_rosbag2_directory(p) {
                out.push(p.to_path_buf());
            }
        }
    } else {
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let p = entry.path();
            if is_rosbag2_directory(&p) {
                out.push(p);
            }
        }
    }

    out.sort();
    out.dedup();
    Ok(out)
}
