mod file;
mod tag;

use crate::error::RepoError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::{io, path};

pub const DATA_DIR_NAME: &str = ".tagfs";
const DB_FILENAME: &str = "tagfs.db";
const TABLES: &str = "
    CREATE TABLE IF NOT EXISTS tracked_files (
        file_id TEXT NOT NULL,
        createTs INTEGER NOT NULL,
        path TEXT NOT NULL,
        PRIMARY KEY (file_id, createTs)
    );
    CREATE TABLE IF NOT EXISTS tags (
        tag_id INTEGER PRIMARY KEY AUTOINCREMENT,  -- Unsigned integer surrogate
        name TEXT UNIQUE NOT NULL
    );
    CREATE TABLE IF NOT EXISTS file_tags (
        file_id TEXT NOT NULL,
        createTs INTEGER NOT NULL,
        tag_id INTEGER NOT NULL,
        PRIMARY KEY (file_id, createTs, tag_id),
        FOREIGN KEY (file_id, createTs) REFERENCES tracked_files(file_id, createTs) ON DELETE CASCADE,
        FOREIGN KEY (tag_id) REFERENCES tags(tag_id) ON DELETE CASCADE
);
    ";
#[derive(Debug)]
pub struct Repo {
    tagfs_dir: PathBuf,
    data_dir: PathBuf,
    conn: Connection,
}
impl Repo {
    pub fn open() -> Result<Self, RepoError> {
        let tagfs_dir = match Self::find_repo_root(None) {
            Ok(root) => root,
            Err(RepoError::RepoNotFound(path)) => path,
            Err(err) => Err(err)?, // Propagate unexpected I/O errors
        };
        let data_dir = Self::data_dir_init(Some(&tagfs_dir))?;
        let conn = Self::open_or_create_db(&data_dir, DB_FILENAME)?;
        Ok(Repo {
            tagfs_dir,
            data_dir,
            conn,
        })
    }
    //helpers
    fn find_repo_root(start_path: Option<&Path>) -> Result<PathBuf, RepoError> {
        let start = match start_path {
            Some(p) => p.to_path_buf(),
            None => std::env::current_dir()?,
        };

        let mut current = start.as_path();

        loop {
            let tagfs_path = current.join(DATA_DIR_NAME);

            if tagfs_path.is_dir() {
                return Ok(current.to_path_buf());
            }

            current = match current.parent() {
                Some(parent) => parent,
                None => return Err(RepoError::RepoNotFound(start)),
            };
        }
    }

    fn data_dir_init(repo_path: Option<&PathBuf>) -> io::Result<PathBuf> {
        let mut path = match repo_path {
            Some(p) => p.clone(),
            None => std::env::current_dir()?,
        };

        path.push(DATA_DIR_NAME);

        if !path.is_dir() {
            std::fs::create_dir_all(&path)?;
        }

        Ok(path)
    }

    fn open_or_create_db(data_dir: &PathBuf, db_filename: &str) -> rusqlite::Result<Connection> {
        let db_path = data_dir.join(db_filename);

        let mut conn: Connection = Connection::open(&db_path)?;
        let tx = conn.transaction()?;
        tx.execute("PRAGMA foreign_keys = ON;", [])?;
        tx.execute_batch(TABLES)?;
        tx.commit()?;
        Ok(conn)
    }
    //tags - quick
    pub fn new_tag(&mut self, tag_name: &str) -> rusqlite::Result<()> {
        crate::repo::tag::new_tag(&mut self.conn, tag_name)
    }
    pub fn update_tag(&mut self, new_name: &str, tag_name: &str) -> rusqlite::Result<Option<()>> {
        crate::repo::tag::update_tag(&mut self.conn, new_name, tag_name)
    }
    pub fn delete_tag(&mut self, tag_name: &str) -> rusqlite::Result<usize> {
        crate::repo::tag::delete_tag(&mut self.conn, tag_name)
    }

    //files - quick
    pub fn new_tracked_file(
        &mut self,
        identifier: &crate::repo::file::TrackedFileUid,
        path: &str,
        tag_names: &[&str],
    ) -> rusqlite::Result<()> {
        crate::repo::file::new_tracked_file(&mut self.conn, identifier, path, tag_names)
    }
    pub fn add_tag_to_Tracked_file(
        &mut self,
        identifier: &crate::repo::file::TrackedFileUid,
        tag_names: &[&str],
    ) -> rusqlite::Result<()> {
        crate::repo::file::add_tags_to_tracked_file(&mut self.conn, identifier, tag_names)
    }
    pub fn update_tracked_file_path(
        &mut self,
        identifier: &crate::repo::file::TrackedFileUid,
        new_path: &str,
    ) -> rusqlite::Result<Option<()>> {
        crate::repo::file::update_tracked_file_path(&mut self.conn, identifier, new_path)
    }
    pub fn delete_tracked_file(
        &mut self,
        identifier: &crate::repo::file::TrackedFileUid,
    ) -> rusqlite::Result<()> {
        crate::repo::file::delete_tracked_file(&mut self.conn, identifier)
    }
    pub fn delete_tag_from_tracked_file(
        &mut self,
        identifier: &crate::repo::file::TrackedFileUid,
        tag_names: &[&str],
    ) -> rusqlite::Result<()> {
        crate::repo::file::delete_tags_from_tracked_file(&mut self.conn, identifier, tag_names)
    }

    //getters
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
    pub fn path(&self) -> &PathBuf {
        &self.tagfs_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_find_repo_root_finds_parent_with_tagfs() {
        let tmp = tempdir().unwrap();
        let parent = tmp.path().join("parent");
        fs::create_dir_all(&parent).unwrap();
        let child = parent.join("child");
        fs::create_dir_all(&child).unwrap();
        // create .tagfs in parent
        let tagfs = parent.join(DATA_DIR_NAME);
        fs::create_dir_all(&tagfs).unwrap();

        let found = Repo::find_repo_root(Some(child.as_path())).unwrap();
        assert_eq!(found, parent);
    }

    #[test]
    fn test_find_repo_root_not_found() {
        let tmp = tempdir().unwrap();
        // no .tagfs created
        let res = Repo::find_repo_root(Some(tmp.path()));
        assert!(res.is_err());
    }

    #[test]
    fn test_data_dir_init_creates_dir() {
        let tmp = tempdir().unwrap();
        let repo_path = tmp.path().join("repo");
        fs::create_dir_all(&repo_path).unwrap();
        let repo_pb = repo_path.clone();
        let data_dir = Repo::data_dir_init(Some(&repo_pb)).unwrap();
        assert!(data_dir.ends_with(DATA_DIR_NAME));
        assert!(data_dir.is_dir());
    }

    #[test]
    fn test_open_or_create_db_creates_db_and_table() {
        let tmp = tempdir().unwrap();
        let data_dir = tmp.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        let conn = Repo::open_or_create_db(&data_dir, DB_FILENAME).unwrap();

        let db_path = data_dir.join(DB_FILENAME);
        assert!(db_path.exists());

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='tracked_files';")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let found = rows.next().unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_new_creates_data_dir_and_db_when_absent() {
        let tmp = tempdir().unwrap();
        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(tmp.path()).unwrap();

        // Ensure no .tagfs initially
        let tagfs_path = tmp.path().join(DATA_DIR_NAME);
        if tagfs_path.exists() {
            fs::remove_dir_all(&tagfs_path).unwrap();
        }

        let repo = Repo::open().unwrap();

        // Repo.path() should be the tmp dir (current dir)
        assert_eq!(repo.path(), &PathBuf::from(tmp.path()));

        // data_dir should exist and be inside tmp
        assert!(repo.data_dir().is_dir());
        assert_eq!(repo.data_dir(), &tagfs_path);

        // DB file should exist
        let db_path = repo.data_dir().join(DB_FILENAME);
        assert!(db_path.exists());

        // restore cwd
        env::set_current_dir(original_cwd).unwrap();
    }
}
