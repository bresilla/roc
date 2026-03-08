pub fn is_hidden_name(name: &str) -> bool {
    name.split('/')
        .filter(|segment| !segment.is_empty())
        .any(|segment| segment.starts_with('_'))
}

pub fn is_hidden_node_name(full_name: &str) -> bool {
    full_name
        .rsplit('/')
        .next()
        .map(|basename| basename.starts_with('_'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{is_hidden_name, is_hidden_node_name};

    #[test]
    fn detects_hidden_ros_names() {
        assert!(is_hidden_name("/_hidden_topic"));
        assert!(is_hidden_name("/demo/_private/service"));
        assert!(is_hidden_name("_hidden_node"));
    }

    #[test]
    fn ignores_visible_ros_names() {
        assert!(!is_hidden_name("/chatter"));
        assert!(!is_hidden_name("/demo/service"));
        assert!(!is_hidden_name("talker"));
    }

    #[test]
    fn detects_hidden_nodes_from_basename_only() {
        assert!(is_hidden_node_name("/_hidden_node"));
        assert!(is_hidden_node_name("/ns/_hidden_node"));
        assert!(!is_hidden_node_name("/_hidden_ns/visible_node"));
        assert!(!is_hidden_node_name("/talker"));
    }
}
