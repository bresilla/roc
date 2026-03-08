use crate::completions::shells::{default_install_path, install_script};
use crate::ui::blocks;
use std::env;
use std::path::PathBuf;

/// Fish completion script with dynamic completions delegated to `roc _complete`.
const SCRIPT: &str = r#"
# Fish completions for roc

function __roc_args
    commandline -opc
end

function __roc_main_cmd
    set -l cmd (__roc_args)
    if test (count $cmd) -ge 2
        echo $cmd[2]
    end
end

function __roc_sub_cmd
    set -l cmd (__roc_args)
    if test (count $cmd) -ge 3
        echo $cmd[3]
    end
end

function __roc_idl_position
    set -l cmd (__roc_args)
    if test (count $cmd) -ge 3
        math (count $cmd) - 2
    else
        echo 1
    end
end

function __roc_idl_args
    set -l cmd (__roc_args)
    if test (count $cmd) -ge 4
        printf '%s\n' $cmd[4..-1]
    end
end

complete -c roc -f -n "not __fish_seen_subcommand_from action topic service param node interface frame run launch work bag daemon middleware idl completion" -a "action topic service param node interface frame run launch work bag daemon middleware idl completion"

complete -c roc -f -n "__fish_seen_subcommand_from launch; and not __roc_sub_cmd" -a "(roc _complete launch '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from launch; and __roc_sub_cmd" -a "(roc _complete launch '' '' 2 (__roc_sub_cmd) 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from run; and not __roc_sub_cmd" -a "(roc _complete run '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from run; and __roc_sub_cmd" -a "(roc _complete run '' '' 2 (__roc_sub_cmd) 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from topic; and not __roc_sub_cmd" -a "(roc _complete topic '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from topic; and __roc_sub_cmd" -a "(roc _complete topic (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub; and test (count (__roc_args)) -ge 4" -a "(roc _complete topic pub '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from service; and not __roc_sub_cmd" -a "(roc _complete service '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from service; and __roc_sub_cmd" -a "(roc _complete service (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = call; and test (count (__roc_args)) -ge 4" -a "(roc _complete service call '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from param; and not __roc_sub_cmd" -a "(roc _complete param '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from param; and __roc_sub_cmd" -a "(roc _complete param (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from param; and contains (__roc_sub_cmd) get set describe remove; and test (count (__roc_args)) -ge 4" -a "(roc _complete param (__roc_sub_cmd) '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from node; and not __roc_sub_cmd" -a "(roc _complete node '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from node; and __roc_sub_cmd" -a "(roc _complete node (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from action; and not __roc_sub_cmd" -a "(roc _complete action '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from action; and __roc_sub_cmd" -a "(roc _complete action (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from action; and test (__roc_sub_cmd) = goal; and test (count (__roc_args)) -ge 4" -a "(roc _complete action goal '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from interface; and not __roc_sub_cmd" -a "(roc _complete interface '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from interface; and __roc_sub_cmd" -a "(roc _complete interface (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from bag; and not __roc_sub_cmd" -a "(roc _complete bag '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from bag; and __roc_sub_cmd" -a "(roc _complete bag (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from work; and not __roc_sub_cmd" -a "(roc _complete work '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from work; and __roc_sub_cmd" -a "(roc _complete work (__roc_sub_cmd) '' 1 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from frame; and not __roc_sub_cmd" -a "(roc _complete frame '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from frame; and __roc_sub_cmd" -a "(roc _complete frame (__roc_sub_cmd) '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from frame; and test (__roc_sub_cmd) = echo; and test (count (__roc_args)) -ge 4" -a "(roc _complete frame echo '' 2 (__roc_args)[4] 2>/dev/null)"

complete -c roc -f -n "__fish_seen_subcommand_from daemon; and not __roc_sub_cmd" -a "(roc _complete daemon '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from middleware; and not __roc_sub_cmd" -a "(roc _complete middleware '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from idl; and not __roc_sub_cmd" -a "(roc _complete idl '' '' 1 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from idl; and __roc_sub_cmd" -a "(roc _complete idl (__roc_sub_cmd) '' (__roc_idl_position) (__roc_idl_args) 2>/dev/null)"
complete -c roc -f -n "__fish_seen_subcommand_from completion; and not __roc_sub_cmd" -a "bash zsh fish"
complete -c roc -f -n "__fish_seen_subcommand_from completion" -l install
complete -c roc -f -n "__fish_seen_subcommand_from completion" -l print-path

complete -c roc -n "__fish_seen_subcommand_from launch" -s n -l noninteractive
complete -c roc -n "__fish_seen_subcommand_from launch" -s d -l debug
complete -c roc -n "__fish_seen_subcommand_from launch" -s p -l print
complete -c roc -n "__fish_seen_subcommand_from launch" -s s -l show_args
complete -c roc -n "__fish_seen_subcommand_from launch" -s a -l show_all
complete -c roc -n "__fish_seen_subcommand_from launch" -l launch_prefix
complete -c roc -n "__fish_seen_subcommand_from launch" -l launch_prefix_filter
complete -c roc -n "__fish_seen_subcommand_from launch" -l output
complete -c roc -n "__fish_seen_subcommand_from run" -l prefix
complete -c roc -n "__fish_seen_subcommand_from run" -l output

complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l base-paths
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l build-base
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l install-base
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l log-base
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-select
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-ignore
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-skip
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-up-to
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-select-build-failed
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-select-build-finished
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-skip-build-finished
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l packages-skip-build-failed
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l parallel-workers
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l merge-install
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l symlink-install
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l cmake-args
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l cmake-target
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l continue-on-error
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l event-handlers
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = build" -l executor
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l base-paths
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l build-base
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l install-base
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l log-base
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l packages-select
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l packages-ignore
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l packages-skip
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l packages-up-to
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l merge-install
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l continue-on-error
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l ctest-args
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test" -l pytest-args
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test-result" -l test-result-base
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test-result" -l all
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test-result" -l verbose
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test-result" -l result-files-only
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test-result" -l delete
complete -c roc -n "__fish_seen_subcommand_from work; and test (__roc_sub_cmd) = test-result" -l delete-yes

complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l qos-profile
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l qos-depth
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l qos-history
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l qos-reliability
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l qos-durability
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l csv
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l field
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -s f -l full-length
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -s l -l truncate-length
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l no-arr
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l no-str
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l flow-style
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l no-lost-messages
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l raw
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = echo" -l once

complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = hz" -s w -l window
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = hz" -l filter
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = hz" -l wall-time
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = info" -s v -l verbose
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = info" -l output
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = list" -s t -l show-types
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = list" -s c -l count-topics
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = list" -s a -l include-hidden-topics
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = list" -l output
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -s r -l rate
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -s p -l print
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l once
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -s t -l times
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l wait-matching-subscriptions
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l keep-alive
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -s n -l node-name
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l qos-profile
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l qos-depth
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l qos-history
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l qos-reliability
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l qos-durability
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = pub" -l output
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = kind" -l output
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = bw" -s w -l window
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = find" -s c -l count-topics
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = find" -s a -l include-hidden-topics
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = find" -l output
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = delay" -s o -l output
complete -c roc -n "__fish_seen_subcommand_from topic; and test (__roc_sub_cmd) = delay" -s v -l verbose
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = find" -s c -l count-services
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = find" -s a -l include-hidden-services
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = find" -l output
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = call" -s r -l rate
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = call" -l output
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = list" -s t -l show-types
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = list" -s c -l count-services
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = list" -s a -l include-hidden-services
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = list" -l output
complete -c roc -n "__fish_seen_subcommand_from action; and test (__roc_sub_cmd) = goal" -s f -l feedback
complete -c roc -n "__fish_seen_subcommand_from action; and test (__roc_sub_cmd) = goal" -l output
complete -c roc -n "__fish_seen_subcommand_from service; and test (__roc_sub_cmd) = kind" -l output
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = get" -s a -l include-hidden-nodes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = get" -l hide-type
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = get" -l output
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = list" -s a -l include-hidden-nodes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = list" -l param-prefixes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = list" -l param-type
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = list" -l filter
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = list" -l output
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = set" -s a -l include-hidden-nodes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = set" -l output
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = export" -s o -l output-dir
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = export" -s a -l include-hidden-nodes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = export" -l output
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = remove" -s a -l include-hidden-nodes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = remove" -l output
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = describe" -s a -l include-hidden-nodes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = describe" -l output
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = import" -l no-use-wildcard
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = import" -s a -l include-hidden-nodes
complete -c roc -n "__fish_seen_subcommand_from param; and test (__roc_sub_cmd) = import" -l output
complete -c roc -n "__fish_seen_subcommand_from bag; and test (__roc_sub_cmd) = list" -l recursive
complete -c roc -n "__fish_seen_subcommand_from bag; and test (__roc_sub_cmd) = list" -l output
complete -c roc -n "__fish_seen_subcommand_from bag; and test (__roc_sub_cmd) = info" -l output
complete -c roc -n "__fish_seen_subcommand_from daemon; and contains (__roc_sub_cmd) start stop status" -l output
complete -c roc -n "__fish_seen_subcommand_from middleware; and test (__roc_sub_cmd) = list" -l output
complete -c roc -n "__fish_seen_subcommand_from middleware; and test (__roc_sub_cmd) = get" -l output
complete -c roc -n "__fish_seen_subcommand_from middleware; and test (__roc_sub_cmd) = set" -l output
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s d -l discover
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s r -l search-root
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -l max-depth
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s o -l output
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s p -l package
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s c -l config
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s I -l include
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s v -l verbose
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) protobuf proto pb" -s n -l dry-run
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) ros2msg msg ros2" -s o -l output
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) ros2msg msg ros2" -s p -l package
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) ros2msg msg ros2" -s v -l verbose
complete -c roc -n "__fish_seen_subcommand_from idl; and contains (__roc_sub_cmd) ros2msg msg ros2" -s n -l dry-run
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = list" -s m -l messages
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = list" -s s -l services
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = list" -s a -l actions
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = list" -l output
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = all" -s m -l messages
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = all" -s s -l services
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = all" -s a -l actions
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = all" -l output
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = package" -l output
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = show" -l all-comments
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = show" -l no-comments
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = show" -l output
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = model" -l no-quotes
complete -c roc -n "__fish_seen_subcommand_from interface; and test (__roc_sub_cmd) = model" -l output
complete -c roc -n "__fish_seen_subcommand_from frame; and test (__roc_sub_cmd) = pub" -l detach
complete -c roc -n "__fish_seen_subcommand_from frame; and test (__roc_sub_cmd) = pub" -l output
"#;

pub fn print_completions() {
    println!("{}", SCRIPT);
}

pub fn print_install_path() {
    match default_install_path(candidate_locations()) {
        Some(path) => println!("{}", path.display()),
        None => {
            blocks::eprint_warning("Could not determine installation path for fish completions")
        }
    }
}

pub fn install_completion() {
    match install_script(SCRIPT, candidate_locations()) {
        Ok(path) => {
            blocks::print_section("COMPLETION");
            blocks::print_field("Shell", "fish");
            blocks::print_field("Path", path.display());
            blocks::print_success("Installed completion script");
            blocks::print_note("Completions should be available in new fish sessions.");
        }
        Err(error) => {
            blocks::eprint_section("COMPLETION");
            blocks::eprint_field("Shell", "fish");
            blocks::eprint_warning(&format!("Failed to install completion script: {error}"));
            blocks::eprint_note("Manual install: roc completion fish > completion_file");
        }
    }
}

fn candidate_locations() -> Vec<Option<PathBuf>> {
    vec![
        env::home_dir().map(|h| h.join(".config/fish/completions/roc.fish")),
        Some(PathBuf::from("/usr/share/fish/completions/roc.fish")),
    ]
}

#[cfg(test)]
mod tests {
    use super::SCRIPT;

    #[test]
    fn fish_script_completes_kind_subcommands() {
        assert!(SCRIPT.contains("roc _complete topic '' '' 1"));
        assert!(SCRIPT.contains("roc _complete service '' '' 1"));
    }

    #[test]
    fn fish_script_supports_install_flag() {
        assert!(SCRIPT.contains("-l install"));
    }

    #[test]
    fn fish_script_completes_work_build_flags() {
        assert!(SCRIPT.contains("-l merge-install"));
        assert!(SCRIPT.contains("-l packages-select"));
    }

    #[test]
    fn fish_script_completes_work_test_flags() {
        assert!(SCRIPT.contains("test (__roc_sub_cmd) = test"));
        assert!(SCRIPT.contains("-l ctest-args"));
        assert!(SCRIPT.contains("-l pytest-args"));
    }

    #[test]
    fn fish_script_completes_work_test_result_flags() {
        assert!(SCRIPT.contains("test (__roc_sub_cmd) = test-result"));
        assert!(SCRIPT.contains("-l delete-yes"));
    }

    #[test]
    fn fish_script_completes_idl_flags() {
        assert!(SCRIPT.contains("roc _complete idl"));
        assert!(SCRIPT.contains("-l search-root"));
        assert!(SCRIPT.contains("__roc_idl_position"));
    }

    #[test]
    fn fish_script_completes_completion_install_flags() {
        assert!(SCRIPT.contains("-l print-path"));
    }
}
