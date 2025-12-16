//use crate::{error::RepoError};
use rusqlite::{Connection, Result as SqlResult, params};
//use std::path::Path;

pub(super) fn new_tag(conn: &mut Connection, tag_name: &str) -> SqlResult<i64> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO tags (name) VALUES (?1) ON CONFLICT(name) DO NOTHING;",
        params![tag_name],
    )?;
    let tag_id: i64 = tx.query_row(
        "SELECT tag_id FROM tags WHERE name = ?1;",
        params![tag_name],
        |row| row.get(0),
    )?;
    tx.commit()?;
    Ok(tag_id)
}
pub(super) fn update_tag(
    conn: &mut Connection,
    new_name: &str,
    previous_name: &str,
) -> SqlResult<Option<i64>> {
    let tx = conn.transaction()?;
    let affected = tx.execute(
        "UPDATE tags SET name = ?1 WHERE name = ?2;",
        params![new_name, previous_name],
    )?;
    if affected == 0 {
        tx.commit()?;
        return Ok(None);
    }
    let tag_id: i64 = tx.query_row(
        "SELECT tag_id FROM tags WHERE name = ?1;",
        params![new_name],
        |row| row.get(0),
    )?;
    tx.commit()?;
    Ok(Some(tag_id))
}
