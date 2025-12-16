use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Repository does not exist at the specified path and it's parent directories: {0}")]
    RepoNotFound(PathBuf),
    #[error("Unable to Create or Access Data Directory .tagfs/: {0}")]
    Disconnect(#[from] io::Error),
    #[error("Unable to initialize database: {0}")]
    Redaction(#[from] rusqlite::Error),
}
