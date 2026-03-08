pub fn is_hidden_name(name: &str) -> bool {
    name.split('/')
        .filter(|segment| !segment.is_empty())
        .any(|segment| segment.starts_with('_'))
}

#[cfg(test)]
mod tests {
    use super::is_hidden_name;

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
}
