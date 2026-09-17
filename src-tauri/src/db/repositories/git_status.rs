use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::core::git::GitStatus;
use crate::errors::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatusEntry {
    pub project_id: i64,
    pub refreshed_at: i64,
    #[serde(flatten)]
    pub status: GitStatus,
}

const COLUMNS: &str = "project_id, branch, ahead, behind, is_dirty, last_commit_sha,
     last_commit_msg, last_commit_at, remote_url, refreshed_at";

fn row_to_entry(row: &Row<'_>) -> rusqlite::Result<GitStatusEntry> {
    Ok(GitStatusEntry {
        project_id: row.get(0)?,
        status: GitStatus {
            branch: row.get(1)?,
            ahead: row.get(2)?,
            behind: row.get(3)?,
            is_dirty: row.get(4)?,
            last_commit_sha: row.get(5)?,
            last_commit_msg: row.get(6)?,
            last_commit_at: row.get(7)?,
            remote_url: row.get(8)?,
        },
        refreshed_at: row.get(9)?,
    })
}

pub fn upsert(
    conn: &Connection,
    project_id: i64,
    status: &GitStatus,
    refreshed_at: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO git_status_cache
            (project_id, branch, ahead, behind, is_dirty, last_commit_sha,
             last_commit_msg, last_commit_at, remote_url, refreshed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(project_id) DO UPDATE SET
            branch = excluded.branch,
            ahead = excluded.ahead,
            behind = excluded.behind,
            is_dirty = excluded.is_dirty,
            last_commit_sha = excluded.last_commit_sha,
            last_commit_msg = excluded.last_commit_msg,
            last_commit_at = excluded.last_commit_at,
            remote_url = excluded.remote_url,
            refreshed_at = excluded.refreshed_at",
        params![
            project_id,
            status.branch,
            status.ahead,
            status.behind,
            status.is_dirty,
            status.last_commit_sha,
            status.last_commit_msg,
            status.last_commit_at,
            status.remote_url,
            refreshed_at
        ],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, project_id: i64) -> Result<Option<GitStatusEntry>> {
    let sql = format!("SELECT {COLUMNS} FROM git_status_cache WHERE project_id = ?1");
    Ok(conn
        .query_row(&sql, params![project_id], row_to_entry)
        .optional()?)
}

pub fn list_all(conn: &Connection) -> Result<Vec<GitStatusEntry>> {
    let sql = format!("SELECT {COLUMNS} FROM git_status_cache");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_entry)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn delete(conn: &Connection, project_id: i64) -> Result<bool> {
    let affected = conn.execute(
        "DELETE FROM git_status_cache WHERE project_id = ?1",
        params![project_id],
    )?;
    Ok(affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::project::DiscoveredProject;
    use crate::db::repositories::projects as projects_repo;
    use crate::db::Db;

    fn sample_status() -> GitStatus {
        GitStatus {
            branch: Some("main".into()),
            ahead: Some(2),
            behind: Some(1),
            is_dirty: true,
            last_commit_sha: Some("abc123".into()),
            last_commit_msg: Some("feat: algo".into()),
            last_commit_at: Some(1_700_000_000),
            remote_url: Some("https://example.invalid/repo.git".into()),
        }
    }

    fn project_with_id(db: &Db) -> i64 {
        let path = std::path::PathBuf::from("/code/mosaic");
        let discovered = DiscoveredProject {
            name: "mosaic".into(),
            path,
            is_git_repo: true,
            primary_language: Some("Rust".into()),
            markers: vec![".git".into()],
        };
        db.with_conn(|conn| projects_repo::upsert_by_path(conn, &discovered, 1_000))
            .unwrap()
            .id
    }

    #[test]
    fn upsert_then_get_roundtrips() {
        let db = Db::open_in_memory().unwrap();
        let id = project_with_id(&db);
        let status = sample_status();

        db.with_conn(|conn| upsert(conn, id, &status, 5_000))
            .unwrap();
        let stored = db.with_conn(|conn| get(conn, id)).unwrap().unwrap();

        assert_eq!(stored.project_id, id);
        assert_eq!(stored.refreshed_at, 5_000);
        assert_eq!(stored.status, status);
    }

    #[test]
    fn upsert_replaces_the_previous_row() {
        let db = Db::open_in_memory().unwrap();
        let id = project_with_id(&db);

        db.with_conn(|conn| upsert(conn, id, &sample_status(), 5_000))
            .unwrap();

        let limpio = GitStatus {
            is_dirty: false,
            ..sample_status()
        };
        db.with_conn(|conn| upsert(conn, id, &limpio, 6_000))
            .unwrap();

        let all = db.with_conn(list_all).unwrap();
        assert_eq!(all.len(), 1, "no se duplica por proyecto");
        assert!(!all[0].status.is_dirty);
        assert_eq!(all[0].refreshed_at, 6_000);
    }

    #[test]
    fn get_returns_none_without_cache() {
        let db = Db::open_in_memory().unwrap();
        let id = project_with_id(&db);

        assert!(db.with_conn(|conn| get(conn, id)).unwrap().is_none());
    }

    #[test]
    fn delete_removes_the_cached_row() {
        let db = Db::open_in_memory().unwrap();
        let id = project_with_id(&db);
        db.with_conn(|conn| upsert(conn, id, &sample_status(), 5_000))
            .unwrap();

        assert!(db.with_conn(|conn| delete(conn, id)).unwrap());
        assert!(db.with_conn(|conn| get(conn, id)).unwrap().is_none());
        assert!(!db.with_conn(|conn| delete(conn, id)).unwrap());
    }

    #[test]
    fn cache_is_removed_with_its_project() {
        let db = Db::open_in_memory().unwrap();
        let id = project_with_id(&db);
        db.with_conn(|conn| upsert(conn, id, &sample_status(), 5_000))
            .unwrap();

        db.with_conn(|conn| {
            conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
            Ok(())
        })
        .unwrap();

        assert!(
            db.with_conn(list_all).unwrap().is_empty(),
            "la foreign key con ON DELETE CASCADE debe limpiar la caché"
        );
    }
}
