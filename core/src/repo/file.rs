use rusqlite::{Connection, Result as SqlResult, params};

#[derive(Debug)]
pub struct TrackedFileUid {
    file_id: String,
    create_ts: i64,
}

impl TrackedFileUid {
    pub fn new(file_id: &str, create_ts: i64) -> Self {
        Self {
            file_id: file_id.to_string(),
            create_ts,
        }
    }
    //read only getters
    pub fn file_id(&self) -> &str {
        &self.file_id
    }
    pub fn create_ts(&self) -> &i64 {
        &self.create_ts
    }
}

/// Creates a new tracked file with associated tags, tags are created if they don't already exist.
pub(super) fn new_tracked_file(
    conn: &mut Connection,
    identifier: &TrackedFileUid,
    path: &str,
    tag_names: &[&str],
) -> SqlResult<()> {
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT OR IGNORE INTO tracked_files (file_id, createTs, path) VALUES (?1, ?2, ?3)",
        params![identifier.file_id(), identifier.create_ts(), path],
    )?;

    for &tag_name in tag_names {
        // Upsert tag (Future might make it return error if tag doesn't exist?)
        tx.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![tag_name],
        )?;

        let tag_id: i64 = tx.query_row(
            "SELECT tag_id FROM tags WHERE name = ?1",
            params![tag_name],
            |row| row.get(0),
        )?;

        tx.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, createTs, tag_id) VALUES (?1, ?2, ?3)",
            params![identifier.file_id(), identifier.create_ts(), tag_id],
        )?;
    }

    tx.commit()
}

pub(super) fn add_tags_to_tracked_file(
    conn: &mut Connection,
    identifier: &TrackedFileUid,
    tag_names: &[&str],
) -> SqlResult<()> {
    let tx = conn.transaction()?;

    // Upsert tag
    for &tag_name in tag_names {
        // Upsert tag (Future might make it return error if tag doesn't exist?)
        tx.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![tag_name],
        )?;

        let tag_id: i64 = tx.query_row(
            "SELECT tag_id FROM tags WHERE name = ?1",
            params![tag_name],
            |row| row.get(0),
        )?;

        tx.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, createTs, tag_id) VALUES (?1, ?2, ?3)",
            params![identifier.file_id(), identifier.create_ts(), tag_id],
        )?;
    }

    tx.commit()
}

pub(super) fn update_tracked_file_path(
    conn: &mut Connection,
    identifier: &TrackedFileUid,
    new_path: &str,
) -> SqlResult<Option<()>> {
    let tx = conn.transaction()?;
    let committed = tx.execute(
        "UPDATE tracked_files SET path = ?1 WHERE file_id = ?2 AND createTs = ?3",
        params![new_path, identifier.file_id(), identifier.create_ts()],
    )?;
    if committed == 0 {
        return Ok(None);
    }
    tx.commit()?;
    Ok(Some(()))
}

pub(super) fn delete_tracked_file(
    conn: &mut Connection,
    identifier: &TrackedFileUid,
) -> SqlResult<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM tracked_files WHERE file_id = ?1 AND createTs = ?2",
        params![identifier.file_id(), identifier.create_ts()],
    )?;
    tx.commit()
}

pub(super) fn delete_tags_from_tracked_file(
    conn: &mut Connection,
    identifier: &TrackedFileUid,
    tag_names: &[&str],
) -> SqlResult<()> {
    let tx = conn.transaction()?;

    for &tag_name in tag_names {
        let tag_id: i64 = tx.query_row(
            "SELECT tag_id FROM tags WHERE name = ?1",
            params![tag_name],
            |row| row.get(0),
        )?;

        tx.execute(
            "DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
            params![identifier.file_id(), tag_id],
        )?;
    }

    tx.commit()
}
