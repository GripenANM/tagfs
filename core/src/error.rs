//use crate::repo::TableName;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Repository does not exist at the specified path and it's parent directories: {0}")]
    RepoNotFound(PathBuf),
    #[error("i/o Error: {0}")]
    IO(#[from] io::Error),
    #[error("database Error: {0}")]
    Database(#[from] rusqlite::Error),
}
