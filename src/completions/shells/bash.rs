use crate::completions::shells::{default_install_path, install_script};
use crate::ui::blocks;
use std::env;
use std::path::PathBuf;

/// Bash completion script with dynamic completions delegated to `roc _complete`.
const SCRIPT: &str = r#"
_roc_completion() {
    local cur prev words cword
    _init_completion || return

    local top="${words[1]}"
    local run_flags="--prefix --output"
    local launch_flags="-n --noninteractive -d --debug -p --print -s --show_args -a --show_all --launch_prefix --launch_prefix_filter --output"
    local work_build_flags="--base-paths --build-base --install-base --log-base --packages-select --packages-ignore --packages-skip --packages-up-to --packages-select-build-failed --packages-select-build-finished --packages-skip-build-finished --packages-skip-build-failed --parallel-workers --merge-install --symlink-install --cmake-args --cmake-target --continue-on-error --event-handlers --executor"
    local work_test_flags="--base-paths --build-base --install-base --log-base --packages-select --packages-ignore --packages-skip --packages-up-to --merge-install --continue-on-error --ctest-args --pytest-args"
    local work_test_result_flags="--test-result-base --all --verbose --result-files-only --delete --delete-yes"
    local topic_echo_flags="--qos-profile --qos-depth --qos-history --qos-reliability --qos-durability --csv --field -f --full-length -l --truncate-length --no-arr --no-str --flow-style --no-lost-messages --raw --once"
    local topic_hz_flags="-w --window --filter --wall-time"
    local topic_info_flags="-v --verbose --output"
    local topic_list_flags="-t --show-types -c --count-topics -a --include-hidden-topics --output"
    local topic_pub_flags="-r --rate -p --print --once -1 -t --times --wait-matching-subscriptions --keep-alive -n --node-name --qos-profile --qos-depth --qos-history --qos-reliability --qos-durability --output"
    local topic_kind_flags="--output"
    local topic_bw_flags="-w --window"
    local topic_find_flags="-c --count-topics -a --include-hidden-topics --output"
    local topic_delay_flags="-o --output -v --verbose"
    local frame_pub_flags="--detach --output"
    local service_call_flags="-r --rate --output"
    local service_find_flags="-c --count-services -a --include-hidden-services --output"
    local service_list_flags="-t --show-types -c --count-services -a --include-hidden-services --output"
    local service_kind_flags="--output"
    local action_goal_flags="-f --feedback --output"
    local param_get_flags="-a --include-hidden-nodes --hide-type --output"
    local param_list_flags="-a --include-hidden-nodes --param-prefixes --param-type --filter --output"
    local param_set_flags="-a --include-hidden-nodes --output"
    local param_export_flags="-o --output-dir -a --include-hidden-nodes --output"
    local param_remove_flags="-a --include-hidden-nodes --output"
    local param_describe_flags="-a --include-hidden-nodes --output"
    local param_import_flags="--no-use-wildcard -a --include-hidden-nodes --output"
    local bag_list_flags="--recursive --output"
    local bag_info_flags="--output"
    local interface_list_flags="-m --messages -s --services -a --actions --output"
    local interface_all_flags="-m --messages -s --services -a --actions --output"
    local interface_package_flags="--output"
    local interface_show_flags="--all-comments --no-comments --output"
    local interface_model_flags="--no-quotes --output"
    local daemon_flags="--output"
    local middleware_list_flags="--output"
    local middleware_get_flags="--output"
    local middleware_set_flags="--output"
    local idl_protobuf_flags="-d --discover -r --search-root --max-depth -o --output -p --package -c --config -I --include -v --verbose -n --dry-run"
    local idl_ros2msg_flags="-o --output -p --package -v --verbose -n --dry-run"

    if [[ "$cur" == -* ]]; then
        case "$top" in
            launch)
                COMPREPLY=($(compgen -W "$launch_flags" -- "$cur"))
                return
                ;;
            run)
                COMPREPLY=($(compgen -W "$run_flags" -- "$cur"))
                return
                ;;
            work)
                if [[ "${words[2]}" == "build" ]]; then
                    COMPREPLY=($(compgen -W "$work_build_flags" -- "$cur"))
                    return
                elif [[ "${words[2]}" == "test" ]]; then
                    COMPREPLY=($(compgen -W "$work_test_flags" -- "$cur"))
                    return
                elif [[ "${words[2]}" == "test-result" ]]; then
                    COMPREPLY=($(compgen -W "$work_test_result_flags" -- "$cur"))
                    return
                fi
                ;;
            topic)
                case "${words[2]}" in
                    echo) COMPREPLY=($(compgen -W "$topic_echo_flags" -- "$cur")); return ;;
                    hz) COMPREPLY=($(compgen -W "$topic_hz_flags" -- "$cur")); return ;;
                    info) COMPREPLY=($(compgen -W "$topic_info_flags" -- "$cur")); return ;;
                    list) COMPREPLY=($(compgen -W "$topic_list_flags" -- "$cur")); return ;;
                    pub) COMPREPLY=($(compgen -W "$topic_pub_flags" -- "$cur")); return ;;
                    kind) COMPREPLY=($(compgen -W "$topic_kind_flags" -- "$cur")); return ;;
                    bw) COMPREPLY=($(compgen -W "$topic_bw_flags" -- "$cur")); return ;;
                    find) COMPREPLY=($(compgen -W "$topic_find_flags" -- "$cur")); return ;;
                    delay) COMPREPLY=($(compgen -W "$topic_delay_flags" -- "$cur")); return ;;
                esac
                ;;
            frame)
                case "${words[2]}" in
                    pub) COMPREPLY=($(compgen -W "$frame_pub_flags" -- "$cur")); return ;;
                esac
                ;;
            service)
                case "${words[2]}" in
                    call) COMPREPLY=($(compgen -W "$service_call_flags" -- "$cur")); return ;;
                    find) COMPREPLY=($(compgen -W "$service_find_flags" -- "$cur")); return ;;
                    list) COMPREPLY=($(compgen -W "$service_list_flags" -- "$cur")); return ;;
                    kind) COMPREPLY=($(compgen -W "$service_kind_flags" -- "$cur")); return ;;
                esac
                ;;
            action)
                case "${words[2]}" in
                    goal) COMPREPLY=($(compgen -W "$action_goal_flags" -- "$cur")); return ;;
                esac
                ;;
            param)
                case "${words[2]}" in
                    get) COMPREPLY=($(compgen -W "$param_get_flags" -- "$cur")); return ;;
                    list) COMPREPLY=($(compgen -W "$param_list_flags" -- "$cur")); return ;;
                    set) COMPREPLY=($(compgen -W "$param_set_flags" -- "$cur")); return ;;
                    export) COMPREPLY=($(compgen -W "$param_export_flags" -- "$cur")); return ;;
                    remove) COMPREPLY=($(compgen -W "$param_remove_flags" -- "$cur")); return ;;
                    describe) COMPREPLY=($(compgen -W "$param_describe_flags" -- "$cur")); return ;;
                    import) COMPREPLY=($(compgen -W "$param_import_flags" -- "$cur")); return ;;
                esac
                ;;
            bag)
                case "${words[2]}" in
                    list) COMPREPLY=($(compgen -W "$bag_list_flags" -- "$cur")); return ;;
                    info) COMPREPLY=($(compgen -W "$bag_info_flags" -- "$cur")); return ;;
                esac
                ;;
            daemon)
                case "${words[2]}" in
                    start|stop|status) COMPREPLY=($(compgen -W "$daemon_flags" -- "$cur")); return ;;
                esac
                ;;
            middleware)
                case "${words[2]}" in
                    list) COMPREPLY=($(compgen -W "$middleware_list_flags" -- "$cur")); return ;;
                    get) COMPREPLY=($(compgen -W "$middleware_get_flags" -- "$cur")); return ;;
                    set) COMPREPLY=($(compgen -W "$middleware_set_flags" -- "$cur")); return ;;
                esac
                ;;
            idl)
                case "${words[2]}" in
                    protobuf|proto|pb) COMPREPLY=($(compgen -W "$idl_protobuf_flags" -- "$cur")); return ;;
                    ros2msg|msg|ros2) COMPREPLY=($(compgen -W "$idl_ros2msg_flags" -- "$cur")); return ;;
                esac
                ;;
            interface)
                case "${words[2]}" in
                    list) COMPREPLY=($(compgen -W "$interface_list_flags" -- "$cur")); return ;;
                    all) COMPREPLY=($(compgen -W "$interface_all_flags" -- "$cur")); return ;;
                    package) COMPREPLY=($(compgen -W "$interface_package_flags" -- "$cur")); return ;;
                    show) COMPREPLY=($(compgen -W "$interface_show_flags" -- "$cur")); return ;;
                    model) COMPREPLY=($(compgen -W "$interface_model_flags" -- "$cur")); return ;;
                esac
                ;;
        esac
    fi

    case "$top" in
        "")
            COMPREPLY=($(compgen -W "action topic service param node interface frame run launch work bag daemon middleware idl completion" -- "$cur"))
            ;;
        launch)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete launch '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete launch '' '' 2 "${words[2]}" 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        run)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete run '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete run '' '' 2 "${words[2]}" 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        topic)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete topic '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete topic "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "pub" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete topic pub '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        service)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete service '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete service "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "call" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete service call '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        param)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete param '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete param "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    case "${words[2]}" in
                        get|set|describe|remove)
                            COMPREPLY=($(compgen -W "$(roc _complete param "${words[2]}" '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                            ;;
                    esac
                    ;;
            esac
            ;;
        node)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete node '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete node "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        action)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete action '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete action "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "goal" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete action goal '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        interface)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete interface '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete interface "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        bag)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete bag '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete bag "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        work)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete work '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete work "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
            esac
            ;;
        frame)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "$(roc _complete frame '' '' 1 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -W "$(roc _complete frame "${words[2]}" '' 1 2>/dev/null)" -- "$cur")) ;;
                4)
                    if [[ "${words[2]}" == "echo" ]]; then
                        COMPREPLY=($(compgen -W "$(roc _complete frame echo '' 2 "${words[3]}" 2>/dev/null)" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        daemon)
            if [[ "$cword" == 2 ]]; then
                COMPREPLY=($(compgen -W "$(roc _complete daemon '' '' 1 2>/dev/null)" -- "$cur"))
            fi
            ;;
        middleware)
            if [[ "$cword" == 2 ]]; then
                COMPREPLY=($(compgen -W "$(roc _complete middleware '' '' 1 2>/dev/null)" -- "$cur"))
            fi
            ;;
        idl)
            case "$cword" in
                2)
                    COMPREPLY=($(compgen -W "$(roc _complete idl '' '' 1 2>/dev/null)" -- "$cur"))
                    ;;
                *)
                    local idl_position=$((cword - 2))
                    local idl_args=()
                    if (( cword > 3 )); then
                        idl_args=("${words[@]:3:$((cword - 3))}")
                    fi
                    COMPREPLY=($(compgen -W "$(roc _complete idl "${words[2]}" '' "$idl_position" "${idl_args[@]}" 2>/dev/null)" -- "$cur"))
                    ;;
            esac
            ;;
        completion)
            case "$cword" in
                2) COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur")) ;;
                *) COMPREPLY=($(compgen -W "--install --print-path" -- "$cur")) ;;
            esac
            ;;
    esac
}

complete -F _roc_completion roc
"#;

pub fn print_completions() {
    println!("{}", SCRIPT);
}

pub fn print_install_path() {
    match default_install_path(candidate_locations()) {
        Some(path) => println!("{}", path.display()),
        None => {
            blocks::eprint_warning("Could not determine installation path for bash completions")
        }
    }
}

pub fn install_completion() {
    match install_script(SCRIPT, candidate_locations()) {
        Ok(path) => {
            blocks::print_section("COMPLETION");
            blocks::print_field("Shell", "bash");
            blocks::print_field("Path", path.display());
            blocks::print_success("Installed completion script");
            blocks::print_note(
                "Add this to ~/.bashrc if your shell does not load it automatically.",
            );
            blocks::print_field("Source", format!("source {}", path.display()));
        }
        Err(error) => {
            blocks::eprint_section("COMPLETION");
            blocks::eprint_field("Shell", "bash");
            blocks::eprint_warning(&format!("Failed to install completion script: {error}"));
            blocks::eprint_note("Manual install: roc completion bash > completion_file");
        }
    }
}

fn candidate_locations() -> Vec<Option<PathBuf>> {
    vec![
        env::home_dir().map(|h| h.join(".local/share/bash-completion/completions/roc")),
        env::home_dir().map(|h| h.join(".bash_completion.d/roc")),
        Some(PathBuf::from("/usr/share/bash-completion/completions/roc")),
    ]
}

#[cfg(test)]
mod tests {
    use super::SCRIPT;

    #[test]
    fn bash_script_uses_kind_not_type() {
        assert!(SCRIPT.contains("roc _complete topic"));
        assert!(!SCRIPT.contains("topic \"${words[2]}\" \"\" 1 $current_args"));
    }

    #[test]
    fn bash_script_completes_service_call_type_position() {
        assert!(SCRIPT.contains("roc _complete service call '' 2"));
    }

    #[test]
    fn bash_script_completes_work_build_flags() {
        assert!(SCRIPT.contains("local work_build_flags="));
        assert!(SCRIPT.contains("--merge-install"));
        assert!(SCRIPT.contains("--packages-select"));
    }

    #[test]
    fn bash_script_completes_work_test_flags() {
        assert!(SCRIPT.contains("local work_test_flags="));
        assert!(SCRIPT.contains("--ctest-args"));
        assert!(SCRIPT.contains("--pytest-args"));
    }

    #[test]
    fn bash_script_completes_work_test_result_flags() {
        assert!(SCRIPT.contains("local work_test_result_flags="));
        assert!(SCRIPT.contains("--delete-yes"));
    }

    #[test]
    fn bash_script_completes_launch_flags() {
        assert!(SCRIPT.contains("local launch_flags="));
        assert!(SCRIPT.contains("--launch_prefix"));
        assert!(SCRIPT.contains("--show_args"));
    }

    #[test]
    fn bash_script_completes_idl_flags() {
        assert!(SCRIPT.contains("local idl_protobuf_flags="));
        assert!(SCRIPT.contains("--search-root"));
        assert!(SCRIPT.contains("roc _complete idl"));
    }

    #[test]
    fn bash_script_completes_completion_install_flags() {
        assert!(SCRIPT.contains("--install --print-path"));
    }
}
