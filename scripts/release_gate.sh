#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES_WS="${ROC_VALIDATION_ROS2_EXAMPLES:-/tmp/roc_ros2_examples}"
DEMOS_WS="${ROC_VALIDATION_ROS2_DEMOS:-/tmp/roc_ros2_demos}"

require_command() {
    local name="$1"
    if ! command -v "$name" >/dev/null 2>&1; then
        echo "Missing required command: $name" >&2
        exit 1
    fi
}

require_path() {
    local path="$1"
    if [[ ! -e "$path" ]]; then
        echo "Missing required path: $path" >&2
        exit 1
    fi
}

echo "ROC release gate (Linux/Jazzy)"
echo "Examples workspace: $EXAMPLES_WS"
echo "Demos workspace:    $DEMOS_WS"

require_command cargo
require_command colcon
require_path /opt/ros/jazzy/setup.bash
require_path "$EXAMPLES_WS"
require_path "$DEMOS_WS"

cd "$ROOT_DIR"

echo
echo "[1/2] Running repository test suite"
cargo test

echo
echo "[2/2] Running ignored real-workspace validation suite"
ROC_VALIDATION_ROS2_EXAMPLES="$EXAMPLES_WS" \
ROC_VALIDATION_ROS2_DEMOS="$DEMOS_WS" \
cargo test --test real_workspace_validation -- --ignored --nocapture

echo
echo "Release gate passed."
echo "Update COMPAT_VALIDATION.md with the validation date and outcome for this run before release."
