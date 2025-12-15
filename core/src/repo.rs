use std::io;
use std::path::PathBuf;

use rusqlite::{Connection, Result};

pub const DATA_DIR_NAME: &str = ".tagfs";
const DB_FILENAME: &str = "tagfs.db";
const TABLES: &str = "
        CREATE TABLE IF NOT EXISTS tracked_files (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            path          TEXT NOT NULL,
            file_id       TEXT NOT NULL,  
            last_modified INTEGER NOT NULL,
            created_at    INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
    ";

/// Ensures the hidden directory exists and returns its path.
fn data_dir(dir_name: &str) -> io::Result<PathBuf> {
    let hidden_name = dir_name.to_string();

    let mut path = std::env::current_dir()?;
    path.push(hidden_name);

    if !path.is_dir() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}
fn initialize_database(data_dir: PathBuf, db_filename: &str) -> Result<Connection> {
    let db_path = data_dir.join(db_filename);

    let conn: Connection = Connection::open(&db_path)?;
    //conn.execute("PRAGMA journal_mode=WAL;", [])?;
    match conn.execute_batch(TABLES) {
        Ok(_) => (),
        Err(e) => {
            return Err(e);
        }
    }
    Ok(conn)
}

pub fn init() -> Result<Connection, Box<dyn std::error::Error>> {
    let data_dir = data_dir(&DATA_DIR_NAME)?;
    let conn = initialize_database(data_dir, DB_FILENAME)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_ensure_hidden_dir_creates_if_missing() {
        let test_dir_name = ".tagfs";

        let _ = fs::remove_dir_all(test_dir_name);

        let result_path = data_dir(test_dir_name).expect("Failed to create directory");

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

        let path1 = data_dir(test_dir_name).unwrap();
        assert!(path1.exists());

        let path2 = data_dir(test_dir_name).unwrap();
        assert_eq!(path1, path2);

        fs::remove_dir_all(test_dir_name).unwrap();
    }
}
