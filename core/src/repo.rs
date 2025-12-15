use std::io;
use std::path::PathBuf;

/// Ensures the hidden directory exists and returns its path.
fn ensure_hidden_dir(dir_name: &str) -> io::Result<PathBuf> {
    let hidden_name = if dir_name.starts_with('.') {
        dir_name.to_string()
    } else {
        format!(".{}", dir_name)
    };

    let mut path = std::env::current_dir()?;
    path.push(hidden_name);

    if !path.is_dir() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}

pub fn init() -> io::Result<PathBuf> {
    ensure_hidden_dir(".tagfs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_ensure_hidden_dir_creates_if_missing() {
        let test_dir_name = ".tagfs";

        let _ = fs::remove_dir_all(test_dir_name);

        let result_path = ensure_hidden_dir(test_dir_name).expect("Failed to create directory");

        assert!(
            result_path.exists(),
            "Directory should exist after creation"
        );
        assert!(result_path.is_dir(), "Path should be a directory");
        assert_eq!(
            result_path.file_name().unwrap().to_str().unwrap(),
            test_dir_name,
            "Directory name should match (including leading dot)"
        );

        fs::remove_dir_all(result_path).expect("Failed to clean up test directory");
    }

    #[test]
    fn test_ensure_hidden_dir_idempotent() {
        let test_dir_name = ".tagfs";

        let _ = fs::remove_dir_all(test_dir_name);

        let path1 = ensure_hidden_dir(test_dir_name).unwrap();
        assert!(path1.exists());

        let path2 = ensure_hidden_dir(test_dir_name).unwrap();
        assert_eq!(path1, path2);

        fs::remove_dir_all(test_dir_name).unwrap();
    }
}
