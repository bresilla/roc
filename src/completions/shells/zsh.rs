use crate::completions::shells::{default_install_path, install_script};
use crate::ui::blocks;
use std::env;
use std::path::PathBuf;

/// Zsh completion script with corrected word indexing and dynamic dispatch.
const SCRIPT: &str = r#"
#compdef roc

_roc_dynamic_lines() {
    local -a items
    items=("${(@f)$("$@" 2>/dev/null)}")
    print -l -- $items
}

_roc() {
    local curcontext="$curcontext" state line
    typeset -A opt_args
    local -a launch_flags work_build_flags work_test_flags work_test_result_flags topic_echo_flags topic_hz_flags topic_info_flags topic_list_flags topic_pub_flags topic_kind_flags topic_bw_flags topic_find_flags topic_delay_flags service_find_flags service_list_flags service_kind_flags param_get_flags param_list_flags param_set_flags param_export_flags param_remove_flags param_describe_flags param_import_flags bag_list_flags bag_info_flags daemon_flags middleware_list_flags middleware_get_flags middleware_set_flags idl_protobuf_flags idl_ros2msg_flags interface_list_flags interface_all_flags interface_package_flags interface_show_flags interface_model_flags
    launch_flags=(-n --noninteractive -d --debug -p --print -s --show_args -a --show_all --launch_prefix --launch_prefix_filter)
    work_build_flags=(--base-paths --build-base --install-base --log-base --packages-select --packages-ignore --packages-skip --packages-up-to --packages-select-build-failed --packages-select-build-finished --packages-skip-build-finished --packages-skip-build-failed --parallel-workers --merge-install --symlink-install --cmake-args --cmake-target --continue-on-error --event-handlers --executor)
    work_test_flags=(--base-paths --build-base --install-base --log-base --packages-select --packages-ignore --packages-skip --packages-up-to --merge-install --continue-on-error --ctest-args --pytest-args)
    work_test_result_flags=(--test-result-base --all --verbose --result-files-only --delete --delete-yes)
    topic_echo_flags=(--qos-profile --qos-depth --qos-history --qos-reliability --qos-durability --csv --field -f --full-length -l --truncate-length --no-arr --no-str --flow-style --no-lost-messages --raw --once)
    topic_hz_flags=(-w --window --filter --wall-time)
    topic_info_flags=(-v --verbose --output)
    topic_list_flags=(-t --show-types -c --count-topics -a --include-hidden-topics --output)
    topic_pub_flags=(-r --rate -p --print --once -1 -t --times --wait-matching-subscriptions --keep-alive -n --node-name --qos-profile --qos-depth --qos-history --qos-reliability --qos-durability)
    topic_kind_flags=(--output)
    topic_bw_flags=(-w --window)
    topic_find_flags=(-c --count-topics -a --include-hidden-topics --output)
    topic_delay_flags=(-o --output -v --verbose)
    service_find_flags=(-c --count-services -a --include-hidden-services --output)
    service_list_flags=(-t --show-types -c --count-services -a --include-hidden-services --output)
    service_kind_flags=(--output)
    param_get_flags=(-a --include-hidden-nodes --hide-type --output)
    param_list_flags=(-a --include-hidden-nodes --param-prefixes --param-type --filter --output)
    param_set_flags=(-a --include-hidden-nodes --output)
    param_export_flags=(-o --output-dir -a --include-hidden-nodes --output)
    param_remove_flags=(-a --include-hidden-nodes --output)
    param_describe_flags=(-a --include-hidden-nodes --output)
    param_import_flags=(--no-use-wildcard -a --include-hidden-nodes --output)
    bag_list_flags=(--recursive --output)
    bag_info_flags=(--output)
    daemon_flags=(--output)
    middleware_list_flags=(--output)
    middleware_get_flags=(--output)
    middleware_set_flags=(--output)
    idl_protobuf_flags=(-d --discover -r --search-root --max-depth -o --output -p --package -c --config -I --include -v --verbose -n --dry-run)
    idl_ros2msg_flags=(-o --output -p --package -v --verbose -n --dry-run)
    interface_list_flags=(-m --messages -s --services -a --actions --output)
    interface_all_flags=(-m --messages -s --services -a --actions --output)
    interface_package_flags=(--output)
    interface_show_flags=(--all-comments --no-comments --output)
    interface_model_flags=(--no-quotes --output)

    if [[ "$words[$CURRENT]" == -* ]]; then
        case "$words[2]" in
            launch) _describe 'launch flags' launch_flags; return ;;
            work)
                if [[ "$words[3]" == "build" ]]; then
                    _describe 'work build flags' work_build_flags
                    return
                elif [[ "$words[3]" == "test" ]]; then
                    _describe 'work test flags' work_test_flags
                    return
                elif [[ "$words[3]" == "test-result" ]]; then
                    _describe 'work test-result flags' work_test_result_flags
                    return
                fi
                ;;
            topic)
                case "$words[3]" in
                    echo) _describe 'topic echo flags' topic_echo_flags; return ;;
                    hz) _describe 'topic hz flags' topic_hz_flags; return ;;
                    info) _describe 'topic info flags' topic_info_flags; return ;;
                    list) _describe 'topic list flags' topic_list_flags; return ;;
                    pub) _describe 'topic pub flags' topic_pub_flags; return ;;
                    kind) _describe 'topic kind flags' topic_kind_flags; return ;;
                    bw) _describe 'topic bw flags' topic_bw_flags; return ;;
                    find) _describe 'topic find flags' topic_find_flags; return ;;
                    delay) _describe 'topic delay flags' topic_delay_flags; return ;;
                esac
                ;;
            service)
                case "$words[3]" in
                    find) _describe 'service find flags' service_find_flags; return ;;
                    list) _describe 'service list flags' service_list_flags; return ;;
                    kind) _describe 'service kind flags' service_kind_flags; return ;;
                esac
                ;;
            param)
                case "$words[3]" in
                    get) _describe 'param get flags' param_get_flags; return ;;
                    list) _describe 'param list flags' param_list_flags; return ;;
                    set) _describe 'param set flags' param_set_flags; return ;;
                    export) _describe 'param export flags' param_export_flags; return ;;
                    remove) _describe 'param remove flags' param_remove_flags; return ;;
                    describe) _describe 'param describe flags' param_describe_flags; return ;;
                    import) _describe 'param import flags' param_import_flags; return ;;
                esac
                ;;
            bag)
                case "$words[3]" in
                    list) _describe 'bag list flags' bag_list_flags; return ;;
                    info) _describe 'bag info flags' bag_info_flags; return ;;
                esac
                ;;
            daemon)
                case "$words[3]" in
                    start|stop|status) _describe 'daemon flags' daemon_flags; return ;;
                esac
                ;;
            middleware)
                case "$words[3]" in
                    list) _describe 'middleware list flags' middleware_list_flags; return ;;
                    get) _describe 'middleware get flags' middleware_get_flags; return ;;
                    set) _describe 'middleware set flags' middleware_set_flags; return ;;
                esac
                ;;
            idl)
                case "$words[3]" in
                    protobuf|proto|pb) _describe 'idl protobuf flags' idl_protobuf_flags; return ;;
                    ros2msg|msg|ros2) _describe 'idl ros2msg flags' idl_ros2msg_flags; return ;;
                esac
                ;;
            interface)
                case "$words[3]" in
                    list) _describe 'interface list flags' interface_list_flags; return ;;
                    all) _describe 'interface all flags' interface_all_flags; return ;;
                    package) _describe 'interface package flags' interface_package_flags; return ;;
                    show) _describe 'interface show flags' interface_show_flags; return ;;
                    model) _describe 'interface model flags' interface_model_flags; return ;;
                esac
                ;;
        esac
    fi

    _arguments -C \
        '1:command:->command' \
        '*::arg:->args'

    case "$state" in
        command)
            local commands=(
                'action:Various action subcommands'
                'topic:Various topic subcommands'
                'service:Various service subcommands'
                'param:Various param subcommands'
                'node:Various node subcommands'
                'interface:Various interface subcommands'
                'frame:Various transform subcommands'
                'run:Run an executable'
                'launch:Launch a launch file'
                'work:Packages and workspace'
                'bag:ROS bag tools'
                'daemon:Daemon and bridge'
                'middleware:Middleware settings'
                'idl:Interface definition tools'
                'completion:Generate shell completions'
            )
            _describe 'command' commands
            return
            ;;
        args)
            case "$words[2]" in
                launch)
                    case $CURRENT in
                        3) _describe 'packages' "$(_roc_dynamic_lines roc _complete launch '' '' 1)" ;;
                        4) _describe 'launch files' "$(_roc_dynamic_lines roc _complete launch '' '' 2 "$words[3]")" ;;
                    esac
                    ;;
                run)
                    case $CURRENT in
                        3) _describe 'packages' "$(_roc_dynamic_lines roc _complete run '' '' 1)" ;;
                        4) _describe 'executables' "$(_roc_dynamic_lines roc _complete run '' '' 2 "$words[3]")" ;;
                    esac
                    ;;
                topic)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete topic '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete topic "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "pub" ]]; then
                                _describe 'message types' "$(_roc_dynamic_lines roc _complete topic pub '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                service)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete service '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete service "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "call" ]]; then
                                _describe 'service types' "$(_roc_dynamic_lines roc _complete service call '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                param)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete param '' '' 1)" ;;
                        4) _describe 'nodes' "$(_roc_dynamic_lines roc _complete param "$words[3]" '' 1)" ;;
                        5)
                            case "$words[3]" in
                                get|set|describe|remove)
                                    _describe 'parameters' "$(_roc_dynamic_lines roc _complete param "$words[3]" '' 2 "$words[4]")"
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                node)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete node '' '' 1)" ;;
                        4) _describe 'nodes' "$(_roc_dynamic_lines roc _complete node "$words[3]" '' 1)" ;;
                    esac
                    ;;
                action)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete action '' '' 1)" ;;
                        4) _describe 'actions' "$(_roc_dynamic_lines roc _complete action "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "goal" ]]; then
                                _describe 'action types' "$(_roc_dynamic_lines roc _complete action goal '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                interface)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete interface '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete interface "$words[3]" '' 1)" ;;
                    esac
                    ;;
                bag)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete bag '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete bag "$words[3]" '' 1)" ;;
                    esac
                    ;;
                work)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete work '' '' 1)" ;;
                        4) _describe 'values' "$(_roc_dynamic_lines roc _complete work "$words[3]" '' 1)" ;;
                    esac
                    ;;
                frame)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete frame '' '' 1)" ;;
                        4) _describe 'frames' "$(_roc_dynamic_lines roc _complete frame "$words[3]" '' 1)" ;;
                        5)
                            if [[ "$words[3]" == "echo" ]]; then
                                _describe 'frames' "$(_roc_dynamic_lines roc _complete frame echo '' 2 "$words[4]")"
                            fi
                            ;;
                    esac
                    ;;
                daemon)
                    [[ $CURRENT -eq 3 ]] && _describe 'subcommands' "$(_roc_dynamic_lines roc _complete daemon '' '' 1)"
                    ;;
                middleware)
                    [[ $CURRENT -eq 3 ]] && _describe 'subcommands' "$(_roc_dynamic_lines roc _complete middleware '' '' 1)"
                    ;;
                idl)
                    case $CURRENT in
                        3) _describe 'subcommands' "$(_roc_dynamic_lines roc _complete idl '' '' 1)" ;;
                        *)
                            local -a idl_args
                            if (( CURRENT > 4 )); then
                                idl_args=("${(@)words[4,$((CURRENT-1))]}")
                            else
                                idl_args=()
                            fi
                            _describe 'values' "$(_roc_dynamic_lines roc _complete idl "$words[3]" '' $((CURRENT-3)) ${(@)idl_args})"
                            ;;
                    esac
                    ;;
                completion)
                    case $CURRENT in
                        3) _describe 'shells' "bash zsh fish" ;;
                        *) _arguments \
                            '--install[Install completions to a default location]' \
                            '--print-path[Print the default installation path]' ;;
                    esac
                    ;;
            esac
            ;;
    esac
}

compdef _roc roc
"#;

pub fn print_completions() {
    println!("{}", SCRIPT);
}

pub fn print_install_path() {
    match default_install_path(candidate_locations()) {
        Some(path) => println!("{}", path.display()),
        None => blocks::eprint_warning("Could not determine installation path for zsh completions"),
    }
}

pub fn install_completion() {
    match install_script(SCRIPT, candidate_locations()) {
        Ok(path) => {
            blocks::print_section("COMPLETION");
            blocks::print_field("Shell", "zsh");
            blocks::print_field("Path", path.display());
            blocks::print_success("Installed completion script");
            if path
                .parent()
                .and_then(|parent| parent.to_str())
                .unwrap_or("")
                .contains(".zfunc")
            {
                blocks::print_field("fpath", "fpath=(~/.zfunc $fpath)");
            }
            blocks::print_field("Setup", "autoload -U compinit && compinit");
        }
        Err(error) => {
            blocks::eprint_section("COMPLETION");
            blocks::eprint_field("Shell", "zsh");
            blocks::eprint_warning(&format!("Failed to install completion script: {error}"));
            blocks::eprint_note("Manual install: roc completion zsh > completion_file");
        }
    }
}

fn candidate_locations() -> Vec<Option<PathBuf>> {
    vec![
        env::home_dir().map(|h| h.join(".zfunc/_roc")),
        env::home_dir().map(|h| h.join(".local/share/zsh/site-functions/_roc")),
        Some(PathBuf::from("/usr/local/share/zsh/site-functions/_roc")),
    ]
}

#[cfg(test)]
mod tests {
    use super::SCRIPT;

    #[test]
    fn zsh_script_uses_correct_word_indexing() {
        assert!(SCRIPT.contains("case \"$words[2]\""));
        assert!(!SCRIPT.contains("case $words[1]"));
    }

    #[test]
    fn zsh_script_completes_kind_subcommands() {
        assert!(SCRIPT.contains("roc _complete topic '' '' 1"));
        assert!(SCRIPT.contains("roc _complete service '' '' 1"));
    }

    #[test]
    fn zsh_script_completes_work_build_flags() {
        assert!(SCRIPT.contains("work_build_flags=("));
        assert!(SCRIPT.contains("--continue-on-error"));
    }

    #[test]
    fn zsh_script_completes_work_test_flags() {
        assert!(SCRIPT.contains("work_test_flags=("));
        assert!(SCRIPT.contains("--ctest-args"));
        assert!(SCRIPT.contains("--pytest-args"));
    }

    #[test]
    fn zsh_script_completes_work_test_result_flags() {
        assert!(SCRIPT.contains("work_test_result_flags=("));
        assert!(SCRIPT.contains("--delete-yes"));
    }

    #[test]
    fn zsh_script_completes_idl_flags() {
        assert!(SCRIPT.contains("idl_protobuf_flags=("));
        assert!(SCRIPT.contains("--search-root"));
        assert!(SCRIPT.contains("roc _complete idl"));
    }

    #[test]
    fn zsh_script_completes_completion_install_flags() {
        assert!(SCRIPT.contains("--print-path[Print the default installation path]"));
    }
}
