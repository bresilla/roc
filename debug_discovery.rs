use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let search_path = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let path = PathBuf::from(search_path);
    
    println!("Searching in: {}", path.display());
    println!("Exists: {}", path.exists());
    println!("Is dir: {}", path.is_dir());
    
    println!("\nWalkdir results:");
    for entry in WalkDir::new(&path).into_iter() {
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();
                println!("  Found: {}", entry_path.display());
                
                // Check for package.xml
                let package_xml = entry_path.join("package.xml");
                if package_xml.is_file() {
                    println!("    -> HAS package.xml!");
                }
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
    }
}
