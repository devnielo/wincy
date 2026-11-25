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
    /// Global shortcut that pastes this (pinned) clip.
    pub shortcut: Option<String>,
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
pub(crate) fn migrate(conn: &Connection) -> Result<()> {
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
    if version < 3 {
        // Same downgrade case as above: rerunning this step must be harmless.
        let has_shortcut: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('clips') WHERE name = 'shortcut'",
            [],
            |r| r.get(0),
        )?;
        let alter = if has_shortcut {
            ""
        } else {
            "ALTER TABLE clips ADD COLUMN shortcut TEXT;"
        };
        conn.execute_batch(&format!(
            "BEGIN;
             {alter}
             CREATE UNIQUE INDEX IF NOT EXISTS clips_shortcut ON clips (shortcut);
             CREATE TABLE IF NOT EXISTS snippets (
                 id          INTEGER PRIMARY KEY,
                 name        TEXT NOT NULL,
                 body        TEXT NOT NULL,
                 shortcut    TEXT UNIQUE,
                 created_at  INTEGER NOT NULL,
                 updated_at  INTEGER NOT NULL
             );
             CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
                 text, ocr_text, source_app,
                 content='clips', content_rowid='id', tokenize='unicode61 remove_diacritics 2'
             );
             -- Without it a deleted clip's words stay in old index segments until a merge.
             INSERT INTO clips_fts (clips_fts, rank) VALUES ('secure-delete', 1);
             CREATE TRIGGER IF NOT EXISTS clips_fts_insert AFTER INSERT ON clips BEGIN
                 INSERT INTO clips_fts (rowid, text, ocr_text, source_app)
                     VALUES (new.id, new.text, new.ocr_text, new.source_app);
             END;
             CREATE TRIGGER IF NOT EXISTS clips_fts_delete AFTER DELETE ON clips BEGIN
                 INSERT INTO clips_fts (clips_fts, rowid, text, ocr_text, source_app)
                     VALUES ('delete', old.id, old.text, old.ocr_text, old.source_app);
             END;
             CREATE TRIGGER IF NOT EXISTS clips_fts_update
                 AFTER UPDATE OF text, ocr_text, source_app ON clips BEGIN
                 INSERT INTO clips_fts (clips_fts, rowid, text, ocr_text, source_app)
                     VALUES ('delete', old.id, old.text, old.ocr_text, old.source_app);
                 INSERT INTO clips_fts (rowid, text, ocr_text, source_app)
                     VALUES (new.id, new.text, new.ocr_text, new.source_app);
             END;
             INSERT INTO clips_fts (clips_fts) VALUES ('rebuild');
             PRAGMA user_version = 3;
             COMMIT;"
        ))?;
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

/// Unpinning also clears the clip's shortcut.
pub fn toggle_pin(conn: &Connection, id: i64) -> Result<bool> {
    conn.query_row(
        "UPDATE clips SET pinned = 1 - pinned, shortcut = CASE WHEN pinned THEN NULL ELSE shortcut END
         WHERE id = ?1 RETURNING pinned",
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
     source_app, pinned, created_at, updated_at, use_count, hash, substr(ocr_text, 1, 500), shortcut";

/// All pinned clips plus the newest `unpinned` others; pinned first, then by `updated_at` desc.
pub fn list(conn: &Connection, images_dir: &Path, unpinned: u32) -> Result<Vec<ClipSummary>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SUMMARY_COLUMNS} FROM clips WHERE pinned = 1 OR id IN
             (SELECT id FROM clips WHERE pinned = 0 ORDER BY updated_at DESC LIMIT ?1)
         ORDER BY pinned DESC, updated_at DESC"
    ))?;
    stmt.query_map(params![unpinned], |r| summary(r, images_dir))?
        .collect()
}

/// Splits a search query into an FTS5 MATCH expression (every word quoted as a prefix
/// term, so user input is never parsed as FTS syntax) and lowercased `app:` terms.
pub fn parse_search(query: &str) -> (String, Vec<String>) {
    let mut words = Vec::new();
    let mut apps = Vec::new();
    for token in query.split_whitespace() {
        if token
            .get(..4)
            .is_some_and(|p| p.eq_ignore_ascii_case("app:"))
        {
            // Like the frontend's parseQuery: `app:chrome.exe` means `app:chrome`.
            let term = token[4..].to_lowercase();
            let term = term.strip_suffix(".exe").unwrap_or(&term);
            if !term.is_empty() {
                apps.push(term.to_string());
            }
        // Punctuation-only words have no tokens to match.
        } else if token.chars().any(char::is_alphanumeric) {
            words.push(format!("\"{}\"*", token.replace('"', "\"\"")));
        }
    }
    (words.join(" "), apps)
}

/// Full-text search for histories larger than what `list(.., loaded)` loads. Clips outside
/// that window come first: the loaded ones are mostly found by the frontend already.
pub fn search(
    conn: &Connection,
    images_dir: &Path,
    query: &str,
    limit: u32,
    loaded: u32,
) -> Result<Vec<ClipSummary>> {
    use rusqlite::types::Value;
    let (fts, apps) = parse_search(query);
    let mut sql = format!("SELECT {SUMMARY_COLUMNS} FROM clips WHERE 1");
    let mut args = Vec::new();
    if !fts.is_empty() {
        sql += " AND id IN (SELECT rowid FROM clips_fts WHERE clips_fts MATCH ?)";
        args.push(Value::Text(fts));
    }
    if !apps.is_empty() {
        // `app:` terms are OR-ed and match the exe name without `.exe`.
        sql += " AND EXISTS (SELECT 1 FROM json_each(?) WHERE instr(lower(CASE
                 WHEN source_app LIKE '%.exe' THEN substr(source_app, 1, length(source_app) - 4)
                 ELSE source_app END), value) > 0)";
        args.push(Value::Text(
            serde_json::to_string(&apps).unwrap_or_default(),
        ));
    }
    if args.is_empty() {
        return Ok(Vec::new());
    }
    sql += " ORDER BY pinned = 1 OR id IN
                 (SELECT id FROM clips WHERE pinned = 0 ORDER BY updated_at DESC LIMIT ?),
             pinned DESC, updated_at DESC LIMIT ?";
    args.push(Value::Integer(loaded.into()));
    args.push(Value::Integer(limit.into()));
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_map(rusqlite::params_from_iter(args), |r| summary(r, images_dir))?
        .collect()
}

pub fn get(conn: &Connection, id: i64, images_dir: &Path) -> Result<Option<Clip>> {
    conn.query_row(
        &format!("SELECT {SUMMARY_COLUMNS}, text, html, ocr_text FROM clips WHERE id = ?1"),
        params![id],
        |r| {
            Ok(Clip {
                summary: ClipSummary {
                    ocr_text: r.get(17)?,
                    ..summary(r, images_dir)?
                },
                text: r.get(15)?,
                html: r.get(16)?,
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
        shortcut: r.get(14)?,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutTarget {
    Clip(i64),
    Snippet(i64),
}

/// Every stored pin and snippet shortcut.
pub fn shortcuts(conn: &Connection) -> Result<Vec<(String, ShortcutTarget)>> {
    let mut stmt = conn.prepare(
        "SELECT shortcut, id, 0 FROM clips WHERE shortcut IS NOT NULL AND pinned = 1
         UNION ALL SELECT shortcut, id, 1 FROM snippets WHERE shortcut IS NOT NULL",
    )?;
    stmt.query_map([], |r| {
        let id = r.get(1)?;
        let target = if r.get::<_, bool>(2)? {
            ShortcutTarget::Snippet(id)
        } else {
            ShortcutTarget::Clip(id)
        };
        Ok((r.get(0)?, target))
    })?
    .collect()
}

/// Returns false when the clip does not exist or is not pinned.
pub fn set_clip_shortcut(conn: &Connection, id: i64, shortcut: Option<&str>) -> Result<bool> {
    conn.execute(
        "UPDATE clips SET shortcut = ?2 WHERE id = ?1 AND pinned = 1",
        params![id, shortcut],
    )
    .map(|n| n > 0)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub id: i64,
    pub name: String,
    pub body: String,
    pub shortcut: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

const SNIPPET_COLUMNS: &str = "id, name, body, shortcut, created_at, updated_at";

fn snippet_row(r: &Row) -> Result<Snippet> {
    Ok(Snippet {
        id: r.get(0)?,
        name: r.get(1)?,
        body: r.get(2)?,
        shortcut: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
    })
}

pub fn snippets(conn: &Connection) -> Result<Vec<Snippet>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SNIPPET_COLUMNS} FROM snippets ORDER BY name COLLATE NOCASE, id"
    ))?;
    stmt.query_map([], snippet_row)?.collect()
}

pub fn snippet(conn: &Connection, id: i64) -> Result<Option<Snippet>> {
    conn.query_row(
        &format!("SELECT {SNIPPET_COLUMNS} FROM snippets WHERE id = ?1"),
        params![id],
        snippet_row,
    )
    .optional()
}

/// Inserts (`id = None`) or updates a snippet. `None` when the id does not exist.
pub fn save_snippet(
    conn: &Connection,
    id: Option<i64>,
    name: &str,
    body: &str,
    shortcut: Option<&str>,
) -> Result<Option<Snippet>> {
    let now = now_ms();
    match id {
        None => conn.query_row(
            &format!(
                "INSERT INTO snippets (name, body, shortcut, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4) RETURNING {SNIPPET_COLUMNS}"
            ),
            params![name, body, shortcut, now],
            snippet_row,
        ),
        Some(id) => conn.query_row(
            &format!(
                "UPDATE snippets SET name = ?2, body = ?3, shortcut = ?4, updated_at = ?5
                 WHERE id = ?1 RETURNING {SNIPPET_COLUMNS}"
            ),
            params![id, name, body, shortcut, now],
            snippet_row,
        ),
    }
    .optional()
}

pub fn delete_snippet(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
    Ok(())
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

        let a = list(&conn, Path::new("x"), 2000).unwrap();
        assert_eq!(a.len(), 3);
        let id_c = a.iter().find(|c| c.preview == "text c").unwrap().id;
        assert!(toggle_pin(&conn, id_c).unwrap());

        let (removed, _) = trim(&conn, 1).unwrap();
        assert_eq!(removed, 1);
        let left = list(&conn, Path::new("x"), 2000).unwrap();
        assert_eq!(left.len(), 2);
        assert!(left[0].pinned);

        clear(&conn).unwrap();
        assert_eq!(list(&conn, Path::new("x"), 2000).unwrap().len(), 1);
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
        assert_eq!(version, 3);
        let clips = list(&conn, Path::new("x"), 2000).unwrap();
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
        let img = list(&conn, Path::new("x"), 2000).unwrap().remove(0);
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
        let left: Vec<String> = list(&conn, Path::new("x"), 2000)
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

    /// A v0.2 (user_version 2) database with data.
    fn v2_database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE clips (
                 id INTEGER PRIMARY KEY, hash TEXT NOT NULL UNIQUE, kind TEXT NOT NULL,
                 text TEXT, html TEXT, files TEXT, image_w INTEGER, image_h INTEGER,
                 source_app TEXT, pinned INTEGER NOT NULL DEFAULT 0,
                 created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
                 use_count INTEGER NOT NULL DEFAULT 0, ocr_text TEXT);
             CREATE INDEX clips_order ON clips (pinned, updated_at);
             INSERT INTO clips (hash, kind, text, source_app, pinned, created_at, updated_at) VALUES
                 ('a', 'text', 'Café crème recipe', 'notepad.exe', 0, 1, 1),
                 ('b', 'code', 'fn debounce() {}', 'Code.exe', 1, 2, 2),
                 ('c', 'image', NULL, 'snippingtool.exe', 0, 3, 3);
             UPDATE clips SET ocr_text = 'Invoice total' WHERE hash = 'c';
             PRAGMA user_version = 2;",
        )
        .unwrap();
        conn
    }

    fn ids(clips: &[ClipSummary]) -> Vec<String> {
        let mut v: Vec<String> = clips
            .iter()
            .map(|c| c.source_app.clone().unwrap())
            .collect();
        v.sort();
        v
    }

    #[test]
    fn migrates_v2_database_with_fts_backfill() {
        let conn = v2_database();
        migrate(&conn).unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, 3);
        let x = Path::new("x");
        assert_eq!(list(&conn, x, 2000).unwrap().len(), 3);
        assert!(
            list(&conn, x, 2000)
                .unwrap()
                .iter()
                .all(|c| c.shortcut.is_none())
        );
        assert!(snippets(&conn).unwrap().is_empty());
        // Backfilled rows are searchable: prefix, diacritics, OCR text, source app.
        assert_eq!(
            ids(&search(&conn, x, "cafe", 10, 2000).unwrap()),
            ["notepad.exe"]
        );
        assert_eq!(
            ids(&search(&conn, x, "CREM", 10, 2000).unwrap()),
            ["notepad.exe"]
        );
        assert_eq!(
            ids(&search(&conn, x, "invoice", 10, 2000).unwrap()),
            ["snippingtool.exe"]
        );
        assert_eq!(
            ids(&search(&conn, x, "debounce", 10, 2000).unwrap()),
            ["Code.exe"]
        );
        // Running the step again (downgrade to v0.1 and back) is harmless.
        conn.pragma_update(None, "user_version", 2).unwrap();
        migrate(&conn).unwrap();
        assert_eq!(search(&conn, x, "cafe", 10, 2000).unwrap().len(), 1);
    }

    #[test]
    fn fts_follows_inserts_updates_and_deletes() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let x = Path::new("x");
        upsert(&conn, &text_clip("alpha")).unwrap();
        upsert(&conn, &text_clip("beta")).unwrap();
        assert_eq!(search(&conn, x, "alph", 10, 2000).unwrap().len(), 1);
        assert_eq!(search(&conn, x, "text", 10, 2000).unwrap().len(), 2);
        assert_eq!(search(&conn, x, "text", 1, 2000).unwrap().len(), 1);
        // Words are AND-ed.
        assert_eq!(search(&conn, x, "text beta", 10, 2000).unwrap().len(), 1);
        conn.execute("UPDATE clips SET text = 'gamma' WHERE hash = 'alpha'", [])
            .unwrap();
        assert!(search(&conn, x, "alpha", 10, 2000).unwrap().is_empty());
        assert_eq!(search(&conn, x, "gamma", 10, 2000).unwrap().len(), 1);
        let id = search(&conn, x, "beta", 10, 2000).unwrap()[0].id;
        delete(&conn, id).unwrap();
        assert!(search(&conn, x, "beta", 10, 2000).unwrap().is_empty());
        let integrity: rusqlite::Result<()> = conn
            .execute_batch("INSERT INTO clips_fts (clips_fts, rank) VALUES ('integrity-check', 1)");
        assert!(integrity.is_ok());
    }

    #[test]
    fn search_input_is_never_fts_syntax() {
        let conn = v2_database();
        migrate(&conn).unwrap();
        let x = Path::new("x");
        for q in [
            "\"",
            "\"\"",
            "*",
            "a OR b",
            "NEAR(a b)",
            "col:x",
            "-",
            "(",
            "recipe\"",
            "^cafe",
            "a AND",
        ] {
            assert!(search(&conn, x, q, 10, 2000).is_ok(), "{q}");
        }
        assert!(search(&conn, x, "", 10, 2000).unwrap().is_empty());
        assert!(search(&conn, x, "OR", 10, 2000).unwrap().is_empty());
        assert_eq!(
            parse_search("app:Code  deb\"ounce APP:edge app: -"),
            (
                "\"deb\"\"ounce\"*".to_string(),
                vec!["code".to_string(), "edge".to_string()]
            )
        );
        // app: matches the exe name without ".exe", case-insensitively; terms are OR-ed.
        assert_eq!(
            ids(&search(&conn, x, "app:code", 10, 2000).unwrap()),
            ["Code.exe"]
        );
        assert!(search(&conn, x, "app:exe", 10, 2000).unwrap().is_empty());
        assert_eq!(
            ids(&search(&conn, x, "app:Code.EXE", 10, 2000).unwrap()),
            ["Code.exe"]
        );
        assert_eq!(parse_search("app:Code.exe foo").1, ["code"]);
        assert!(parse_search("app:.exe").1.is_empty());
        assert_eq!(
            ids(&search(&conn, x, "app:note app:snip", 10, 2000).unwrap()),
            ["notepad.exe", "snippingtool.exe"]
        );
        assert!(
            search(&conn, x, "app:code cafe", 10, 2000)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            ids(&search(&conn, x, "app:% cafe", 10, 2000).unwrap()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn list_keeps_pinned_and_newest_unpinned() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        for h in ["1", "2", "3", "4"] {
            upsert(&conn, &text_clip(h)).unwrap();
            conn.execute(
                "UPDATE clips SET updated_at = ?2 WHERE hash = ?1",
                params![h, h.parse::<i64>().unwrap()],
            )
            .unwrap();
        }
        conn.execute("UPDATE clips SET pinned = 1 WHERE hash = '1'", [])
            .unwrap();
        let previews: Vec<String> = list(&conn, Path::new("x"), 2)
            .unwrap()
            .into_iter()
            .map(|c| c.preview)
            .collect();
        assert_eq!(previews, ["text 1", "text 4", "text 3"]);
        // The index returns the clips `list` did not load first.
        let found: Vec<String> = search(&conn, Path::new("x"), "text", 10, 2)
            .unwrap()
            .into_iter()
            .map(|c| c.preview)
            .collect();
        assert_eq!(found, ["text 2", "text 1", "text 4", "text 3"]);
    }

    #[test]
    fn unpinning_clears_the_shortcut() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        upsert(&conn, &text_clip("a")).unwrap();
        let id = list(&conn, Path::new("x"), 10).unwrap()[0].id;
        assert!(!set_clip_shortcut(&conn, id, Some("Alt+Shift+A")).unwrap());
        assert!(toggle_pin(&conn, id).unwrap());
        assert!(set_clip_shortcut(&conn, id, Some("Alt+Shift+A")).unwrap());
        assert_eq!(
            shortcuts(&conn).unwrap(),
            [("Alt+Shift+A".to_string(), ShortcutTarget::Clip(id))]
        );
        assert!(!toggle_pin(&conn, id).unwrap());
        assert!(shortcuts(&conn).unwrap().is_empty());
        assert!(toggle_pin(&conn, id).unwrap());
        assert_eq!(list(&conn, Path::new("x"), 10).unwrap()[0].shortcut, None);
    }

    #[test]
    fn snippet_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let b = save_snippet(&conn, None, "b", "body", Some("Ctrl+Alt+B"))
            .unwrap()
            .unwrap();
        save_snippet(&conn, None, "A", "x", None).unwrap().unwrap();
        let names: Vec<String> = snippets(&conn)
            .unwrap()
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, ["A", "b"]);
        // The unique column rejects a textual duplicate.
        assert!(save_snippet(&conn, None, "c", "x", Some("Ctrl+Alt+B")).is_err());
        let edited = save_snippet(&conn, Some(b.id), "b2", "new", None)
            .unwrap()
            .unwrap();
        assert_eq!((edited.name.as_str(), edited.shortcut), ("b2", None));
        assert_eq!(edited.created_at, b.created_at);
        assert!(
            save_snippet(&conn, Some(999), "x", "y", None)
                .unwrap()
                .is_none()
        );
        delete_snippet(&conn, b.id).unwrap();
        assert!(snippet(&conn, b.id).unwrap().is_none());
    }
}
