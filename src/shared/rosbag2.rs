use anyhow::{anyhow, Result};
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn metadata_yaml() -> &'static str {
        r#"
rosbag2_bagfile_information:
  version: 8
  storage_identifier: mcap
  duration:
    nanoseconds: 123456789
  starting_time:
    nanoseconds_since_epoch: 1700000000000000000
  message_count: 42
  topics_with_message_count:
    - topic_metadata:
        name: /chatter
        type: std_msgs/msg/String
        serialization_format: cdr
      message_count: 40
    - topic_metadata:
        name: /clock
        type: rosgraph_msgs/msg/Clock
        serialization_format: cdr
      message_count: 2
  compression_format: zstd
  compression_mode: file
"#
    }

    #[test]
    fn load_metadata_parses_expected_fields() {
        let temp = tempdir().unwrap();
        std::fs::write(temp.path().join("metadata.yaml"), metadata_yaml()).unwrap();

        let metadata = load_metadata(temp.path()).unwrap();
        let info = metadata.rosbag2_bagfile_information;

        assert_eq!(info.version, 8);
        assert_eq!(info.storage_identifier, "mcap");
        assert_eq!(info.duration.nanoseconds, 123_456_789);
        assert_eq!(info.message_count, 42);
        assert_eq!(info.topics_with_message_count.len(), 2);
        assert_eq!(
            info.topics_with_message_count[0].topic_metadata.name,
            "/chatter"
        );
        assert_eq!(
            info.topics_with_message_count[0].topic_metadata.r#type,
            "std_msgs/msg/String"
        );
        assert_eq!(info.compression_format, "zstd");
        assert_eq!(info.compression_mode, "file");
    }

    #[test]
    fn find_rosbag2_directories_discovers_direct_children_without_recursion() {
        let temp = tempdir().unwrap();
        let bag_a = temp.path().join("bag_a");
        let nested_root = temp.path().join("nested");
        let bag_b = nested_root.join("bag_b");

        std::fs::create_dir_all(&bag_a).unwrap();
        std::fs::create_dir_all(&bag_b).unwrap();
        std::fs::write(bag_a.join("metadata.yaml"), metadata_yaml()).unwrap();
        std::fs::write(bag_b.join("metadata.yaml"), metadata_yaml()).unwrap();

        let found = find_rosbag2_directories(temp.path(), false).unwrap();

        assert_eq!(found, vec![bag_a]);
    }

    #[test]
    fn find_rosbag2_directories_discovers_nested_bags_with_recursion() {
        let temp = tempdir().unwrap();
        let bag_a = temp.path().join("bag_a");
        let nested_root = temp.path().join("nested");
        let bag_b = nested_root.join("bag_b");

        std::fs::create_dir_all(&bag_a).unwrap();
        std::fs::create_dir_all(&bag_b).unwrap();
        std::fs::write(bag_a.join("metadata.yaml"), metadata_yaml()).unwrap();
        std::fs::write(bag_b.join("metadata.yaml"), metadata_yaml()).unwrap();

        let found = find_rosbag2_directories(temp.path(), true).unwrap();

        assert_eq!(found, vec![bag_a, bag_b]);
    }

    #[test]
    fn find_rosbag2_directories_rejects_missing_root() {
        let temp = tempdir().unwrap();
        let missing = temp.path().join("missing");
        let error = find_rosbag2_directories(&missing, true).unwrap_err();
        assert!(error.to_string().contains("Path does not exist"));
    }
}
