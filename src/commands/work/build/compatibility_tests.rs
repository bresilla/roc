use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::commands::work::build::dependency_graph;
use crate::commands::work::build::PackageMeta;
use crate::shared::package_discovery::{discover_packages, DiscoveryConfig};

fn fixture_workspace(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("workspaces")
        .join(name)
}

fn discover_fixture_packages(name: &str) -> Vec<PackageMeta> {
    let workspace_root = fixture_workspace(name);
    let config = DiscoveryConfig {
        base_paths: vec![workspace_root],
        include_hidden: false,
        max_depth: Some(10),
        exclude_patterns: vec![
            "build".to_string(),
            "install".to_string(),
            "log".to_string(),
            ".git".to_string(),
            ".vscode".to_string(),
            "target".to_string(),
            "node_modules".to_string(),
            "__pycache__".to_string(),
        ],
    };

    discover_packages(&config).expect("fixture package discovery should succeed")
}

fn relative_paths(root: &Path) -> BTreeSet<String> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path() != root)
        .map(|entry| {
            entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn assert_paths_exist(root: &Path, expected: &[&str]) {
    let actual = relative_paths(root);
    for path in expected {
        assert!(
            actual.contains(*path),
            "expected fixture path '{}' to exist under '{}'; actual entries: {:?}",
            path,
            root.display(),
            actual
        );
    }
}

#[test]
fn ament_cmake_fixture_is_discoverable() {
    let packages = discover_fixture_packages("ament_cmake_minimal");
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "demo_cmake_pkg");
}

#[test]
fn ament_python_fixture_is_discoverable() {
    let packages = discover_fixture_packages("ament_python_minimal");
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "demo_python_pkg");
}

#[test]
fn dependency_chain_fixture_orders_packages_topologically() {
    let packages = discover_fixture_packages("dependency_chain");
    let order = dependency_graph::topological_sort(&packages).unwrap();

    let ordered_names: Vec<&str> = order.iter().map(|idx| packages[*idx].name.as_str()).collect();
    assert_eq!(
        ordered_names,
        vec!["base_msgs_pkg", "consumer_node_pkg"],
        "dependency chain fixture should establish base -> consumer ordering"
    );
}

#[test]
fn merged_install_fixture_contains_expected_tree_shape() {
    let install_root = fixture_workspace("merged_install_layout").join("install");

    assert_paths_exist(
        &install_root,
        &[
            "setup.bash",
            "bin",
            "bin/demo_merged_node",
            "lib",
            "lib/libdemo_merged.so",
            "share",
            "share/colcon-core",
            "share/colcon-core/packages",
            "share/colcon-core/packages/demo_merged_pkg",
            "share/demo_merged_pkg",
            "share/demo_merged_pkg/package.sh",
            "share/demo_merged_pkg/package.bash",
            "share/demo_merged_pkg/package.zsh",
            "share/demo_merged_pkg/local_setup.sh",
            "share/demo_merged_pkg/local_setup.bash",
            "share/demo_merged_pkg/local_setup.zsh",
            "share/demo_merged_pkg/package.xml",
        ],
    );
}

#[test]
fn overlay_fixture_contains_underlay_and_overlay_workspace_roots() {
    let fixture_root = fixture_workspace("overlay_layout");

    assert_paths_exist(
        &fixture_root,
        &[
            "underlay_ws",
            "underlay_ws/install",
            "underlay_ws/install/setup.bash",
            "underlay_ws/install/share",
            "underlay_ws/install/share/underlay_pkg",
            "overlay_ws",
            "overlay_ws/src",
            "overlay_ws/src/overlay_pkg",
            "overlay_ws/src/overlay_pkg/package.xml",
        ],
    );
}
