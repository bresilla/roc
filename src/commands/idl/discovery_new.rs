use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use crate::shared::package_discovery::{discover_packages, find_idl_files_in_packages, DiscoveryConfig, Package};

#[derive(Debug, Clone)]
pub struct ProjectDiscoveryOptions {
    pub search_root: PathBuf,
    pub include_hidden: bool,
    pub max_depth: Option<usize>,
    pub verbose: bool,
}

impl Default for ProjectDiscoveryOptions {
    fn default() -> Self {
        Self {
            search_root: PathBuf::from("."),
            include_hidden: false,
            max_depth: Some(10), // Reasonable default to avoid infinite recursion
            verbose: false,
        }
    }
}

pub fn discover_idl_projects(options: &ProjectDiscoveryOptions) -> Result<Vec<IdlProject>> {
    if options.verbose {
        println!("🔍 Discovering ROS2 packages in {}...", options.search_root.display());
    }
    
    let config = DiscoveryConfig {
        base_paths: vec![options.search_root.clone()],
        include_hidden: options.include_hidden,
        max_depth: options.max_depth,
        exclude_patterns: vec![
            "build".to_string(),
            "install".to_string(),
            "log".to_string(),
            "target".to_string(),
            ".git".to_string(),
            ".svn".to_string(),
            ".hg".to_string(),
            "node_modules".to_string(),
            ".pytest_cache".to_string(),
            "__pycache__".to_string(),
        ],
    };
    
    let packages = discover_packages(&config)?;
    
    if options.verbose {
        println!("   Found {} ROS2 packages", packages.len());
    }
    
    if packages.is_empty() {
        return Err(anyhow!("No ROS2 packages found in {}", options.search_root.display()));
    }
    
    let idl_files = find_idl_files_in_packages(&packages)?;
    
    let mut projects = Vec::new();
    
    // Group files by package
    for package in packages {
        let mut proto_files = Vec::new();
        let mut msg_files = Vec::new();
        
        // Find proto files in this package
        for proto_file in &idl_files.proto_files {
            if proto_file.starts_with(&package.path) {
                proto_files.push(proto_file.clone());
            }
        }
        
        // Find msg files in this package
        for msg_file in &idl_files.msg_files {
            if msg_file.starts_with(&package.path) {
                msg_files.push(msg_file.clone());
            }
        }
        
        if !proto_files.is_empty() || !msg_files.is_empty() {
            projects.push(IdlProject {
                package_name: package.name.clone(),
                package_path: package.path.clone(),
                proto_files,
                msg_files,
                package: package,
            });
        }
    }
    
    if options.verbose {
        let total_proto = projects.iter().map(|p| p.proto_files.len()).sum::<usize>();
        let total_msg = projects.iter().map(|p| p.msg_files.len()).sum::<usize>();
        println!("   Found {} .proto files and {} .msg files across {} packages",
                 total_proto, total_msg, projects.len());
    }
    
    Ok(projects)
}

#[derive(Debug, Clone)]
pub struct IdlProject {
    pub package_name: String,
    pub package_path: PathBuf,
    pub proto_files: Vec<PathBuf>,
    pub msg_files: Vec<PathBuf>,
    pub package: Package,
}
