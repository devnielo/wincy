//! SQLite storage for clips (`<appDataDir>/wincy.db`).

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, Result, Row, params};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipSummary {
    pub id: i64,
    pub kind: String,
    pub preview: String,
    pub has_html: bool,
    pub thumb_path: Option<String>,
    pub image_path: Option<String>,
    pub image_size: Option<(u32, u32)>,
    pub files: Option<Vec<String>>,
    pub source_app: Option<String>,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub use_count: i64,
    /// Text recognized in an image: `None` until OCR ran, `""` when it found nothing.
    pub ocr_text: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    #[serde(flatten)]
    pub summary: ClipSummary,
    pub text: Option<String>,
    pub html: Option<String>,
}

/// A freshly captured clip, ready to be upserted.
pub struct NewClip {
    pub hash: String,
    pub kind: &'static str,
    pub text: Option<String>,
    pub html: Option<String>,
    pub files: Option<Vec<String>>,
    pub image_size: Option<(u32, u32)>,
    pub source_app: Option<String>,
}

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    // Deleted clips (possibly secrets) must not linger in free pages.
    conn.pragma_update(None, "secure_delete", "ON")?;
    migrate(&conn)?;
    Ok(conn)
}

/// Like `open`, but a corrupt database (and its WAL files) is moved aside as
/// `*.corrupt-<unix secs>` and a fresh one is created, so the app still starts.
/// Other errors (locked, disk full, read-only) are returned: the history is intact.
pub fn open_or_recover(path: &Path) -> Result<Connection> {
    open(path).or_else(|e| {
        if !matches!(
            e.sqlite_error_code(),
            Some(rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase)
        ) {
            return Err(e);
        }
        log::error!("db: {} is unusable ({e}); moving it aside", path.display());
        let ts = now_ms() / 1000;
        for ext in ["", "-wal", "-shm"] {
            let from = format!("{}{ext}", path.display());
            let _ = std::fs::rename(&from, format!("{from}.corrupt-{ts}"));
        }
        open(path)
    })
}

/// Pushes WAL frames into the main file and truncates the WAL, so deleted rows
/// (zeroed by `secure_delete`) leave no plaintext copy behind in `-wal`.
pub fn checkpoint(conn: &Connection) {
    let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
}

// Each step bumps `user_version` in its own transaction. A newer version (downgraded app) is left as is.
fn migrate(conn: &Connection) -> Result<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS clips (
                 id          INTEGER PRIMARY KEY,
                 hash        TEXT NOT NULL UNIQUE,
                 kind        TEXT NOT NULL,
                 text        TEXT,
                 html        TEXT,
                 files       TEXT,
                 image_w     INTEGER,
                 image_h     INTEGER,
                 source_app  TEXT,
                 pinned      INTEGER NOT NULL DEFAULT 0,
                 created_at  INTEGER NOT NULL,
                 updated_at  INTEGER NOT NULL,
                 use_count   INTEGER NOT NULL DEFAULT 0
             );
             CREATE INDEX IF NOT EXISTS clips_order ON clips (pinned, updated_at);
             PRAGMA user_version = 1;
             COMMIT;",
        )?;
    }
    if version < 2 {
        // v0.1.0 rewrote user_version to 1 when it opened a v2 database: the column may exist.
        let has_ocr: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('clips') WHERE name = 'ocr_text'",
            [],
            |r| r.get(0),
        )?;
        let alter = if has_ocr {
            ""
        } else {
            "ALTER TABLE clips ADD COLUMN ocr_text TEXT;"
        };
        conn.execute_batch(&format!("BEGIN; {alter} PRAGMA user_version = 2; COMMIT;"))?;
    }
    Ok(())
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

pub fn image_path(images_dir: &Path, hash: &str) -> PathBuf {
    images_dir.join(format!("{hash}.png"))
}

pub fn thumb_path(images_dir: &Path, hash: &str) -> PathBuf {
    images_dir.join(format!("{hash}.thumb.png"))
}

/// Best effort: a missing file must not block deleting the row.
pub fn remove_images(images_dir: &Path, hashes: &[String]) {
    for h in hashes {
        let _ = std::fs::remove_file(image_path(images_dir, h));
        let _ = std::fs::remove_file(thumb_path(images_dir, h));
    }
}

/// Returns true if this was a new clip, false if an existing one was bumped.
pub fn upsert(conn: &Connection, clip: &NewClip) -> Result<bool> {
    let now = now_ms();
    let bumped = conn.execute(
        "UPDATE clips SET updated_at = ?2, html = ?3, source_app = ?4 WHERE hash = ?1",
        params![clip.hash, now, clip.html, clip.source_app],
    )?;
    if bumped > 0 {
        return Ok(false);
    }
    let files = clip
        .files
        .as_ref()
        .map(|f| serde_json::to_string(f).unwrap_or_default());
    conn.execute(
        "INSERT INTO clips (hash, kind, text, html, files, image_w, image_h, source_app, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            clip.hash,
            clip.kind,
            clip.text,
            clip.html,
            files,
            clip.image_size.map(|s| s.0),
            clip.image_size.map(|s| s.1),
            clip.source_app,
            now
        ],
    )?;
    Ok(true)
}

/// Keeps the newest `max_items` unpinned clips. Returns the hashes of deleted images.
pub fn trim(conn: &Connection, max_items: u32) -> Result<(usize, Vec<String>)> {
    delete_returning(
        conn,
        "DELETE FROM clips WHERE pinned = 0 AND id NOT IN
             (SELECT id FROM clips WHERE pinned = 0 ORDER BY updated_at DESC LIMIT ?1)
         RETURNING kind, hash",
        params![max_items],
    )
}

/// Deletes unpinned clips last used before `cutoff_ms`. Returns the hashes of deleted images.
pub fn expire(conn: &Connection, cutoff_ms: i64) -> Result<(usize, Vec<String>)> {
    delete_returning(
        conn,
        "DELETE FROM clips WHERE pinned = 0 AND updated_at < ?1 RETURNING kind, hash",
        params![cutoff_ms],
    )
}

/// Deletes every unpinned clip. Returns the hashes of deleted images.
pub fn clear(conn: &Connection) -> Result<Vec<String>> {
    delete_returning(
        conn,
        "DELETE FROM clips WHERE pinned = 0 RETURNING kind, hash",
        [],
    )
    .map(|(_, images)| images)
}

pub fn delete(conn: &Connection, id: i64) -> Result<Vec<String>> {
    delete_returning(
        conn,
        "DELETE FROM clips WHERE id = ?1 RETURNING kind, hash",
        params![id],
    )
    .map(|(_, images)| images)
}

fn delete_returning(
    conn: &Connection,
    sql: &str,
    args: impl rusqlite::Params,
) -> Result<(usize, Vec<String>)> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(args, |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>>>()?;
    let count = rows.len();
    let images = rows
        .into_iter()
        .filter(|(kind, _)| kind == "image")
        .map(|(_, hash)| hash)
        .collect();
    Ok((count, images))
}

pub fn toggle_pin(conn: &Connection, id: i64) -> Result<bool> {
    conn.query_row(
        "UPDATE clips SET pinned = 1 - pinned WHERE id = ?1 RETURNING pinned",
        params![id],
        |r| r.get(0),
    )
}

pub fn mark_used(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clips SET updated_at = ?2, use_count = use_count + 1 WHERE id = ?1",
        params![id, now_ms()],
    )?;
    Ok(())
}

// substr counts characters, not bytes, so previews never split a code point.
const SUMMARY_COLUMNS: &str = "id, kind, substr(text, 1, 500), html IS NOT NULL, files, image_w, image_h, \
     source_app, pinned, created_at, updated_at, use_count, hash, substr(ocr_text, 1, 500)";

pub fn list(conn: &Connection, images_dir: &Path) -> Result<Vec<ClipSummary>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SUMMARY_COLUMNS} FROM clips ORDER BY pinned DESC, updated_at DESC"
    ))?;
    stmt.query_map([], |r| summary(r, images_dir))?.collect()
}

pub fn get(conn: &Connection, id: i64, images_dir: &Path) -> Result<Option<Clip>> {
    conn.query_row(
        &format!("SELECT {SUMMARY_COLUMNS}, text, html, ocr_text FROM clips WHERE id = ?1"),
        params![id],
        |r| {
            Ok(Clip {
                summary: ClipSummary {
                    ocr_text: r.get(16)?,
                    ..summary(r, images_dir)?
                },
                text: r.get(14)?,
                html: r.get(15)?,
            })
        },
    )
    .optional()
}

fn summary(r: &Row, images_dir: &Path) -> Result<ClipSummary> {
    let kind: String = r.get(1)?;
    let files: Option<Vec<String>> = r
        .get::<_, Option<String>>(4)?
        .and_then(|j| serde_json::from_str(&j).ok());
    let hash: String = r.get(12)?;
    let is_image = kind == "image";
    let path = |p: PathBuf| is_image.then(|| p.to_string_lossy().into_owned());
    let size = match (r.get::<_, Option<u32>>(5)?, r.get::<_, Option<u32>>(6)?) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    };
    let preview = match &files {
        Some(f) => f.join("\n"),
        None => r.get::<_, Option<String>>(2)?.unwrap_or_default(),
    };
    Ok(ClipSummary {
        id: r.get(0)?,
        thumb_path: path(thumb_path(images_dir, &hash)),
        image_path: path(image_path(images_dir, &hash)),
        kind,
        preview,
        has_html: r.get(3)?,
        image_size: size,
        files,
        source_app: r.get(7)?,
        pinned: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
        use_count: r.get(11)?,
        ocr_text: r.get(13)?,
    })
}

pub fn count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM clips", [], |r| r.get(0))
}

/// Hashes of the most recent image clips still waiting for OCR.
pub fn ocr_pending(conn: &Connection, limit: u32) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT hash FROM clips WHERE kind = 'image' AND ocr_text IS NULL
         ORDER BY updated_at DESC LIMIT ?1",
    )?;
    stmt.query_map(params![limit], |r| r.get(0))?.collect()
}

pub fn needs_ocr(conn: &Connection, hash: &str) -> Result<bool> {
    conn.query_row(
        "SELECT COUNT(*) > 0 FROM clips WHERE hash = ?1 AND kind = 'image' AND ocr_text IS NULL",
        params![hash],
        |r| r.get(0),
    )
}

/// Returns false when the clip was deleted meanwhile.
pub fn set_ocr_text(conn: &Connection, hash: &str, text: &str) -> Result<bool> {
    conn.execute(
        "UPDATE clips SET ocr_text = ?2 WHERE hash = ?1 AND kind = 'image'",
        params![hash, text],
    )
    .map(|n| n > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_clip(hash: &str) -> NewClip {
        NewClip {
            hash: hash.into(),
            kind: "text",
            text: Some(format!("text {hash}")),
            html: None,
            files: None,
            image_size: None,
            source_app: Some("notepad.exe".into()),
        }
    }

    #[test]
    fn upsert_dedupes_and_trim_keeps_pinned() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        for h in ["a", "b", "c"] {
            assert!(upsert(&conn, &text_clip(h)).unwrap());
        }
        assert!(!upsert(&conn, &text_clip("a")).unwrap());

        let a = list(&conn, Path::new("x")).unwrap();
        assert_eq!(a.len(), 3);
        let id_c = a.iter().find(|c| c.preview == "text c").unwrap().id;
        assert!(toggle_pin(&conn, id_c).unwrap());

        let (removed, _) = trim(&conn, 1).unwrap();
        assert_eq!(removed, 1);
        let left = list(&conn, Path::new("x")).unwrap();
        assert_eq!(left.len(), 2);
        assert!(left[0].pinned);

        clear(&conn).unwrap();
        assert_eq!(list(&conn, Path::new("x")).unwrap().len(), 1);
    }

    #[test]
    fn migrates_v1_database_and_survives_downgrade() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE clips (
                 id INTEGER PRIMARY KEY, hash TEXT NOT NULL UNIQUE, kind TEXT NOT NULL,
                 text TEXT, html TEXT, files TEXT, image_w INTEGER, image_h INTEGER,
                 source_app TEXT, pinned INTEGER NOT NULL DEFAULT 0,
                 created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
                 use_count INTEGER NOT NULL DEFAULT 0);
             INSERT INTO clips (hash, kind, text, created_at, updated_at)
                 VALUES ('a', 'text', 'kept', 1, 1), ('img', 'image', NULL, 2, 2);
             PRAGMA user_version = 1;",
        )
        .unwrap();
        migrate(&conn).unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 2);
        let clips = list(&conn, Path::new("x")).unwrap();
        assert_eq!(clips.len(), 2);
        assert!(clips.iter().all(|c| c.ocr_text.is_none()));
        assert!(clips.iter().any(|c| c.preview == "kept"));
        assert_eq!(ocr_pending(&conn, 200).unwrap(), ["img"]);

        // A v0.1.0 build reopening it sets user_version back to 1; the column is already there.
        conn.pragma_update(None, "user_version", 1).unwrap();
        migrate(&conn).unwrap();

        let long = "é".repeat(600);
        assert!(set_ocr_text(&conn, "img", &long).unwrap());
        assert!(!needs_ocr(&conn, "img").unwrap());
        assert!(ocr_pending(&conn, 200).unwrap().is_empty());
        let img = list(&conn, Path::new("x")).unwrap().remove(0);
        assert_eq!(img.ocr_text.unwrap().chars().count(), 500);
        let full = get(&conn, img.id, Path::new("x")).unwrap().unwrap();
        assert_eq!(full.summary.ocr_text.unwrap().chars().count(), 600);
        assert!(!set_ocr_text(&conn, "missing", "x").unwrap());
    }

    #[test]
    fn expire_keeps_pinned_and_recent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        for h in ["old", "old-pinned", "new"] {
            upsert(&conn, &text_clip(h)).unwrap();
        }
        conn.execute(
            "UPDATE clips SET updated_at = 1000 WHERE hash IN ('old', 'old-pinned')",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE clips SET pinned = 1, kind = 'image' WHERE hash = 'old-pinned'",
            [],
        )
        .unwrap();
        conn.execute("UPDATE clips SET kind = 'image' WHERE hash = 'old'", [])
            .unwrap();

        let (removed, images) = expire(&conn, 2000).unwrap();
        assert_eq!((removed, images), (1, vec!["old".to_string()]));
        let left: Vec<String> = list(&conn, Path::new("x"))
            .unwrap()
            .into_iter()
            .map(|c| c.preview)
            .collect();
        assert_eq!(left, ["text old-pinned", "text new"]);
    }
    #[test]
    fn corrupt_database_is_moved_aside() {
        let dir = std::env::temp_dir().join(format!("wincy-db-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("wincy.db");
        std::fs::write(&path, vec![0x42; 4096]).unwrap();
        assert!(open(&path).is_err());

        let conn = open_or_recover(&path).unwrap();
        assert!(upsert(&conn, &text_clip("a")).unwrap());
        let moved = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("wincy.db.corrupt-")
            });
        assert!(moved);
        // Not corruption (here: cannot open): returned, nothing moved aside.
        assert!(open_or_recover(&dir.join("missing").join("wincy.db")).is_err());
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
