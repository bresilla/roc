use std::fs;
use std::io;
use std::path::PathBuf;

pub mod bash;
pub mod fish;
pub mod zsh;

pub fn candidate_paths(locations: Vec<Option<PathBuf>>) -> Vec<PathBuf> {
    locations.into_iter().flatten().collect()
}

pub fn default_install_path(locations: Vec<Option<PathBuf>>) -> Option<PathBuf> {
    candidate_paths(locations).into_iter().next()
}

pub fn install_script(script: &str, locations: Vec<Option<PathBuf>>) -> io::Result<PathBuf> {
    let candidates = candidate_paths(locations);
    let mut last_error = None;

    for path in candidates {
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                last_error = Some(io::Error::new(
                    error.kind(),
                    format!("{}: {}", path.display(), error),
                ));
                continue;
            }
        }

        match fs::write(&path, script) {
            Ok(()) => return Ok(path),
            Err(error) => {
                last_error = Some(io::Error::new(
                    error.kind(),
                    format!("{}: {}", path.display(), error),
                ));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine an installation location",
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::{candidate_paths, default_install_path};
    use std::path::PathBuf;

    #[test]
    fn default_install_path_returns_first_candidate_without_side_effects() {
        let path = default_install_path(vec![
            Some(PathBuf::from("/tmp/first")),
            Some(PathBuf::from("/tmp/second")),
        ]);

        assert_eq!(path, Some(PathBuf::from("/tmp/first")));
    }

    #[test]
    fn candidate_paths_discards_missing_entries() {
        let paths = candidate_paths(vec![
            Some(PathBuf::from("/tmp/first")),
            None,
            Some(PathBuf::from("/tmp/second")),
        ]);

        assert_eq!(
            paths,
            vec![PathBuf::from("/tmp/first"), PathBuf::from("/tmp/second")]
        );
    }
}
