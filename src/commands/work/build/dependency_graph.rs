use std::collections::{HashMap, VecDeque};
use crate::commands::work::build::PackageMeta;

pub fn topological_sort(packages: &[PackageMeta]) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let mut name_to_index: HashMap<String, usize> = HashMap::new();
    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); packages.len()];
    let mut in_degree: Vec<usize> = vec![0; packages.len()];
    
    // Build name to index mapping
    for (idx, package) in packages.iter().enumerate() {
        name_to_index.insert(package.name.clone(), idx);
    }
    
    // Build dependency graph
    for (pkg_idx, package) in packages.iter().enumerate() {
        for dep_name in &package.build_deps {
            if let Some(&dep_idx) = name_to_index.get(dep_name) {
                graph[dep_idx].push(pkg_idx);
                in_degree[pkg_idx] += 1;
            }
            // If dependency not found in workspace, assume it's external (already built)
        }
        
        // Also consider buildtool dependencies for ordering
        for dep_name in &package.buildtool_deps {
            if let Some(&dep_idx) = name_to_index.get(dep_name) {
                graph[dep_idx].push(pkg_idx);
                in_degree[pkg_idx] += 1;
            }
        }
    }
    
    // Kahn's algorithm for topological sorting
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut result: Vec<usize> = Vec::new();
    
    // Add all nodes with no incoming edges
    for (idx, &degree) in in_degree.iter().enumerate() {
        if degree == 0 {
            queue.push_back(idx);
        }
    }
    
    while let Some(current) = queue.pop_front() {
        result.push(current);
        
        // Remove this node and update in-degrees
        for &neighbor in &graph[current] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push_back(neighbor);
            }
        }
    }
    
    // Check for cycles
    if result.len() != packages.len() {
        // Find the cycle
        let remaining: Vec<String> = packages
            .iter()
            .enumerate()
            .filter(|(idx, _)| !result.contains(idx))
            .map(|(_, pkg)| pkg.name.clone())
            .collect();
        
        return Err(format!("Circular dependency detected among packages: {:?}", remaining).into());
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::work::build::{PackageMeta, BuildType};
    use std::path::PathBuf;
    
    fn create_test_package(name: &str, deps: Vec<&str>) -> PackageMeta {
        PackageMeta {
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            build_type: BuildType::AmentCmake,
            version: "1.0.0".to_string(),
            description: "Test package".to_string(),
            maintainers: vec!["test@example.com".to_string()],
            build_deps: deps.iter().map(|s| s.to_string()).collect(),
            buildtool_deps: Vec::new(),
            exec_deps: Vec::new(),
            test_deps: Vec::new(),
        }
    }
    
    #[test]
    fn test_simple_topological_sort() {
        let packages = vec![
            create_test_package("a", vec![]),
            create_test_package("b", vec!["a"]),
            create_test_package("c", vec!["b"]),
        ];
        
        let result = topological_sort(&packages).unwrap();
        
        // a should come before b, b should come before c
        let a_pos = result.iter().position(|&x| packages[x].name == "a").unwrap();
        let b_pos = result.iter().position(|&x| packages[x].name == "b").unwrap();
        let c_pos = result.iter().position(|&x| packages[x].name == "c").unwrap();
        
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }
    
    #[test]
    fn test_circular_dependency() {
        let packages = vec![
            create_test_package("a", vec!["b"]),
            create_test_package("b", vec!["a"]),
        ];
        
        let result = topological_sort(&packages);
        assert!(result.is_err());
    }
}
