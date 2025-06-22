use rclrs::*;

/// Helper function to format topic type hash
pub fn format_topic_type_hash(hash: &rosidl_type_hash_t) -> String {
    // Format the hash as a hexadecimal string
    let hash_bytes = unsafe {
        std::slice::from_raw_parts(hash.value.as_ptr(), hash.value.len())
    };
    hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

/// Helper function to format GID
#[allow(dead_code)]
pub fn format_gid(gid: &[u8]) -> String {
    gid.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join(".")
}