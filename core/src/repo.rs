use std::io;
use std::path::PathBuf;
use thiserror::Error;

use rusqlite::{Connection, Result};

#[derive(Error, Debug)]
pub enum RepoInitError {
    #[error("Unable to Create or Access Data Directory .tagfs/: {0}")]
    Disconnect(#[from] io::Error),
    #[error("Unable to initialize database: {0}")]
    Redaction(#[from] rusqlite::Error),
}

pub const DATA_DIR_NAME: &str = ".tagfs";
const DB_FILENAME: &str = "tagfs.db";
const TABLES: &str = "
    CREATE TABLE IF NOT EXISTS tracked_files (
        file_id TEXT NOT NULL,
        createTs INTEGER NOT NULL,
        path TEXT NOT NULL,
        PRIMARY KEY (file_id, createTs)
    );
    ";

/// Ensures the hidden directory exists and returns its path.
fn data_dir(dir_name: &str) -> io::Result<PathBuf> {
    let dir_name = dir_name.to_string();

    let mut path = std::env::current_dir()?;
    path.push(dir_name);

    if !path.is_dir() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}
fn initialize_database(data_dir: PathBuf, db_filename: &str) -> Result<Connection> {
    let db_path = data_dir.join(db_filename);

    let conn: Connection = Connection::open(&db_path)?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    conn.execute_batch(TABLES)?;
    Ok(conn)
}

pub fn init() -> Result<Connection, RepoInitError> {
    let data_dir = data_dir(&DATA_DIR_NAME)?;
    let conn = initialize_database(data_dir, DB_FILENAME)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
