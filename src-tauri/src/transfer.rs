//! Export / import of the history and snippets (`*.wincy.json`, format version 1).

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use image::ImageFormat;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::clipboard::{self, MAX_HTML_BYTES, MAX_TEXT_BYTES};
use crate::db::{self, NewClip, Snippet};
use crate::{AppState, detect, ocr, shortcuts, snippets};

const FORMAT: &str = "wincy-export";
const VERSION: u32 = 1;
const MAX_FILE_BYTES: u64 = 512 << 20;
const MAX_ITEMS: usize = 100_000;
const MAX_FILES_PER_CLIP: usize = 10_000;
/// Longest Windows path, in UTF-16 units; also bounds `sourceApp`.
const MAX_PATH_LEN: usize = 32_767;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportClip {
    kind: String,
    text: Option<String>,
    html: Option<String>,
    files: Option<Vec<String>>,
    source_app: Option<String>,
    pinned: bool,
    shortcut: Option<String>,
    created_at: i64,
    updated_at: i64,
    use_count: i64,
    ocr_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_png: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportSnippet {
    name: String,
    body: String,
    shortcut: Option<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportFile {
    format: String,
    version: u32,
    exported_at: i64,
    clips: Vec<ExportClip>,
    snippets: Vec<ExportSnippet>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    clips: usize,
    snippets: usize,
}

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    added: usize,
    merged: usize,
    skipped: usize,
    snippets: usize,
}

/// Paths come from the frontend: only absolute `.json` files.
fn check_path(path: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    if p.is_absolute()
        && p.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    {
        Ok(p)
    } else {
        Err("Choose a .wincy.json file".into())
    }
}

pub fn export(app: &AppHandle, path: &str, include_images: bool) -> Result<ExportResult, String> {
    let path = check_path(path)?;
    let state = app.state::<AppState>();
    let (clips, snippets) = {
        let conn = state.db.lock().unwrap();
        (export_clips(&conn), db::snippets(&conn))
    };
    let (clips, snippets) = (clips.map_err(err)?, snippets.map_err(err)?);
    let images = include_images.then_some(state.images_dir.as_path());
    let result = write_file(&path, clips, &snippets, images)
        .map_err(|e| format!("Could not write the file: {e}"))?;
    log::info!(
        "exported {} clips and {} snippets",
        result.clips,
        result.snippets
    );
    Ok(result)
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Every clip with the hash that names its image file.
fn export_clips(conn: &Connection) -> rusqlite::Result<Vec<(String, ExportClip)>> {
    let mut stmt = conn.prepare(
        "SELECT hash, kind, text, html, files, source_app, pinned, shortcut, created_at,
                updated_at, use_count, ocr_text
         FROM clips ORDER BY pinned DESC, updated_at DESC",
    )?;
    stmt.query_map([], |r| {
        let files: Option<String> = r.get(4)?;
        Ok((
            r.get(0)?,
            ExportClip {
                kind: r.get(1)?,
                text: r.get(2)?,
                html: r.get(3)?,
                files: files.and_then(|j| serde_json::from_str(&j).ok()),
                source_app: r.get(5)?,
                pinned: r.get(6)?,
                shortcut: r.get(7)?,
                created_at: r.get(8)?,
                updated_at: r.get(9)?,
                use_count: r.get(10)?,
                ocr_text: r.get(11)?,
                image_png: None,
            },
        ))
    })?
    .collect()
}

/// Streams the file (one image in memory at a time) to a temp file, then renames it.
/// `images`: include image data from this folder.
fn write_file(
    path: &Path,
    clips: Vec<(String, ExportClip)>,
    snippets: &[Snippet],
    images: Option<&Path>,
) -> std::io::Result<ExportResult> {
    let result = ExportResult {
        clips: clips.len(),
        snippets: snippets.len(),
    };
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let done = write_json(&tmp, clips, snippets, images)
        .and_then(|()| {
            // Import refuses larger files: such a backup could never be restored.
            if std::fs::metadata(&tmp)?.len() > MAX_FILE_BYTES {
                Err(std::io::Error::other(
                    "it would be over 512 MB, the import limit; export without images",
                ))
            } else {
                Ok(())
            }
        })
        .and_then(|()| std::fs::rename(&tmp, path));
    if done.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    done.map(|()| result)
}

fn write_json(
    file: &Path,
    clips: Vec<(String, ExportClip)>,
    snippets: &[Snippet],
    images: Option<&Path>,
) -> std::io::Result<()> {
    let mut w = BufWriter::new(File::create(file)?);
    write!(
        w,
        r#"{{"format":"{FORMAT}","version":{VERSION},"exportedAt":{},"clips":["#,
        db::now_ms()
    )?;
    for (i, (hash, mut clip)) in clips.into_iter().enumerate() {
        if i > 0 {
            w.write_all(b",")?;
        }
        if let (Some(dir), "image") = (images, clip.kind.as_str()) {
            clip.image_png = std::fs::read(db::image_path(dir, &hash))
                .ok()
                .map(|png| STANDARD.encode(png));
        }
        serde_json::to_writer(&mut w, &clip)?;
    }
    w.write_all(br#"],"snippets":"#)?;
    let snippets: Vec<ExportSnippet> = snippets
        .iter()
        .map(|s| ExportSnippet {
            name: s.name.clone(),
            body: s.body.clone(),
            shortcut: s.shortcut.clone(),
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
        .collect();
    serde_json::to_writer(&mut w, &snippets)?;
    w.write_all(b"}")?;
    w.into_inner()?.sync_all()
}

/// Reads and checks the envelope. Error messages never quote file contents.
fn read_file(path: &Path, max_bytes: u64) -> Result<ExportFile, String> {
    let file = File::open(path).map_err(|e| format!("Could not open the file: {e}"))?;
    let len = file.metadata().map_err(err)?.len();
    if len > max_bytes {
        return Err(format!(
            "The file is too large (max {} MB)",
            max_bytes >> 20
        ));
    }
    let data: ExportFile =
        serde_json::from_reader(BufReader::new(file.take(max_bytes))).map_err(|e| {
            format!(
                "Not a valid Wincy export (line {}, column {})",
                e.line(),
                e.column()
            )
        })?;
    if data.format != FORMAT || data.version != VERSION || data.exported_at < 0 {
        return Err("Not a Wincy export, or made by a newer version of Wincy".into());
    }
    if data.clips.len() > MAX_ITEMS || data.snippets.len() > MAX_ITEMS {
        return Err(format!("Too many items (max {MAX_ITEMS})"));
    }
    Ok(data)
}

pub fn import(app: &AppHandle, path: &str) -> Result<ImportResult, String> {
    let path = check_path(path)?;
    let file = read_file(&path, MAX_FILE_BYTES)?;
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    // Decoding and saving images happens before taking the database lock.
    let clips = prepare_all(file.clips, settings.ignore_secrets, &state.images_dir);
    let images: Vec<String> = clips
        .iter()
        .flatten()
        .filter(|p| p.clip.kind == "image")
        .map(|p| p.clip.hash.clone())
        .collect();
    let (result, ocr_pending, trimmed) = {
        let mut conn = state.db.lock().unwrap();
        let (result, ocr_pending) = match store(&mut conn, clips, file.snippets, &settings.shortcut)
        {
            Ok(stored) => stored,
            Err(e) => {
                // Rolled back: drop the images written for it, except those an existing clip uses.
                let orphans: Vec<String> = images
                    .into_iter()
                    .filter(|h| {
                        matches!(
                            conn.query_row("SELECT 1 FROM clips WHERE hash = ?1", [h], |_| Ok(()))
                                .optional(),
                            Ok(None)
                        )
                    })
                    .collect();
                db::remove_images(&state.images_dir, &orphans);
                return Err(err(e));
            }
        };
        (result, ocr_pending, db::trim(&conn, settings.max_items))
    };
    match trimmed {
        Ok((_, images)) => db::remove_images(&state.images_dir, &images),
        Err(e) => log::warn!("trim: {e}"),
    }
    if settings.ocr_images {
        ocr_pending.into_iter().for_each(ocr::queue);
    }
    shortcuts::sync(app);
    let _ = app.emit("clips-changed", ());
    let _ = app.emit("snippets-changed", ());
    log::info!("import: {result:?}");
    Ok(result)
}

/// A validated clip from the file, its image (if any) already saved.
struct Prepared {
    clip: NewClip,
    pinned: bool,
    shortcut: Option<String>,
    created_at: i64,
    updated_at: i64,
    use_count: i64,
    ocr_text: Option<String>,
}

/// `None` for each skipped clip.
fn prepare_all(
    clips: Vec<ExportClip>,
    ignore_secrets: bool,
    images_dir: &Path,
) -> Vec<Option<Prepared>> {
    clips
        .into_iter()
        .map(|c| prepare(c, ignore_secrets, images_dir))
        .collect()
}

/// Merges the clips and snippets in one transaction. Returns the counts and the hashes of
/// added images without OCR text.
fn store(
    conn: &mut Connection,
    clips: Vec<Option<Prepared>>,
    snippets: Vec<ExportSnippet>,
    main_shortcut: &str,
) -> rusqlite::Result<(ImportResult, Vec<String>)> {
    let mut result = ImportResult::default();
    let mut ocr_pending = Vec::new();
    let tx = conn.transaction()?;
    for p in clips {
        let Some(p) = p else {
            result.skipped += 1;
            continue;
        };
        let files = p
            .clip
            .files
            .as_ref()
            .map(|f| serde_json::to_string(f).unwrap_or_default());
        let added = tx.execute(
            "INSERT INTO clips (hash, kind, text, html, files, image_w, image_h, source_app,
                                created_at, updated_at, use_count, ocr_text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT (hash) DO NOTHING",
            params![
                p.clip.hash,
                p.clip.kind,
                p.clip.text,
                p.clip.html,
                files,
                p.clip.image_size.map(|s| s.0),
                p.clip.image_size.map(|s| s.1),
                p.clip.source_app,
                p.created_at,
                p.updated_at,
                p.use_count,
                p.ocr_text,
            ],
        )? > 0;
        if added {
            result.added += 1;
            if p.clip.kind == "image" && p.ocr_text.is_none() {
                ocr_pending.push(p.clip.hash.clone());
            }
        } else {
            result.merged += 1;
        }
        // The file's pinned state and shortcut only win over an unpinned clip.
        let pinned = p.pinned
            && tx.execute(
                "UPDATE clips SET pinned = 1 WHERE hash = ?1 AND pinned = 0",
                params![p.clip.hash],
            )? > 0;
        if let (true, Some(text)) = (pinned, p.shortcut)
            && let Ok(text) = shortcuts::validate(&tx, main_shortcut, &text, None)
        {
            tx.execute(
                "UPDATE clips SET shortcut = ?2 WHERE hash = ?1",
                params![p.clip.hash, text],
            )?;
        }
    }
    for s in snippets {
        let Ok(name) = snippets::validate(&s.name, &s.body) else {
            continue;
        };
        let exists: bool = tx.query_row(
            "SELECT COUNT(*) > 0 FROM snippets WHERE name = ?1 AND body = ?2",
            params![name, s.body],
            |r| r.get(0),
        )?;
        if exists {
            continue;
        }
        let shortcut = s
            .shortcut
            .and_then(|t| shortcuts::validate(&tx, main_shortcut, &t, None).ok());
        let (created_at, updated_at) = times(s.created_at, s.updated_at);
        tx.execute(
            "INSERT INTO snippets (name, body, shortcut, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, s.body, shortcut, created_at, updated_at],
        )?;
        result.snippets += 1;
    }
    tx.commit()?;
    Ok((result, ocr_pending))
}

/// Clamped to [0, now] (a future `updatedAt` would stick to the top), created <= updated.
fn times(created_at: i64, updated_at: i64) -> (i64, i64) {
    let updated = updated_at.clamp(0, db::now_ms());
    (created_at.clamp(0, updated), updated)
}

/// `None` = skipped: invalid, a secret (with `ignore_secrets`), or an image without data.
fn prepare(c: ExportClip, ignore_secrets: bool, images_dir: &Path) -> Option<Prepared> {
    let mut clip = NewClip {
        hash: String::new(),
        kind: "text",
        text: None,
        html: None,
        files: None,
        image_size: None,
        source_app: c
            .source_app
            .filter(|s| !s.trim().is_empty() && s.len() <= MAX_PATH_LEN),
    };
    let mut ocr_text = None;
    match c.kind.as_str() {
        "text" | "url" | "color" | "code" => {
            let text = c
                .text
                .filter(|t| !t.trim().is_empty() && t.len() <= MAX_TEXT_BYTES)?;
            if ignore_secrets && detect::looks_secret(&text).is_some() {
                return None;
            }
            clip.kind = detect::detect(&text);
            clip.hash = clipboard::hash(clip.kind, text.as_bytes());
            clip.html = c
                .html
                .filter(|h| h.len() <= MAX_HTML_BYTES && h.starts_with("Version:"));
            clip.text = Some(text);
        }
        "files" => {
            let files = c.files.filter(|f| {
                (1..=MAX_FILES_PER_CLIP).contains(&f.len())
                    && f.iter()
                        .all(|p| !p.is_empty() && p.len() <= MAX_PATH_LEN && !p.contains('\0'))
            })?;
            let mut sorted = files.clone();
            sorted.sort();
            clip.hash = clipboard::hash("files", sorted.join("\n").as_bytes());
            clip.kind = "files";
            clip.files = Some(files);
        }
        "image" => {
            let png = STANDARD.decode(c.image_png?).ok()?;
            let img = image::load_from_memory_with_format(&png, ImageFormat::Png).ok()?;
            clip.hash = clipboard::hash("image", &png);
            clip.kind = "image";
            clip.image_size = Some((img.width(), img.height()));
            if let Err(e) = clipboard::save_image(images_dir, &clip.hash, &png, &img) {
                log::warn!("import: saving image failed: {e}");
                return None;
            }
            ocr_text = c.ocr_text.filter(|t| t.len() <= MAX_TEXT_BYTES);
        }
        _ => return None,
    }
    let (created_at, updated_at) = times(c.created_at, c.updated_at);
    Some(Prepared {
        clip,
        pinned: c.pinned,
        shortcut: c.shortcut,
        created_at,
        updated_at,
        use_count: c.use_count.max(0),
        ocr_text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("wincy-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(dir.join("images")).unwrap();
            Dir(dir)
        }
        fn images(&self) -> PathBuf {
            self.0.join("images")
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn import_into(
        conn: &mut Connection,
        file: ExportFile,
        main_shortcut: &str,
        ignore_secrets: bool,
        images_dir: &Path,
    ) -> rusqlite::Result<(ImportResult, Vec<String>)> {
        let clips = prepare_all(file.clips, ignore_secrets, images_dir);
        store(conn, clips, file.snippets, main_shortcut)
    }

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        conn
    }

    fn png() -> Vec<u8> {
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            2,
            3,
            image::Rgba([1, 2, 3, 255]),
        ))
        .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
        .unwrap();
        out
    }

    fn import(conn: &mut Connection, json: &str, dir: &Dir) -> Result<ImportResult, String> {
        let path = dir.0.join("in.wincy.json");
        std::fs::write(&path, json).unwrap();
        let file = read_file(&path, MAX_FILE_BYTES)?;
        import_into(conn, file, "Alt+V", true, &dir.images())
            .map(|(r, _)| r)
            .map_err(err)
    }

    fn clip_json(kind: &str, extra: &str) -> String {
        format!(
            r#"{{"kind":"{kind}","pinned":false,"createdAt":1,"updatedAt":2,"useCount":0{extra}}}"#
        )
    }

    fn envelope(clips: &[String], snippets: &str) -> String {
        format!(
            r#"{{"format":"wincy-export","version":1,"exportedAt":5,"clips":[{}],"snippets":[{snippets}]}}"#,
            clips.join(",")
        )
    }

    #[test]
    fn round_trip_and_reimport_merges() {
        let src_dir = Dir::new("export-src");
        let dst_dir = Dir::new("export-dst");
        let src = db();
        let png = png();
        let hash = clipboard::hash("image", &png);
        let img = image::load_from_memory(&png).unwrap();
        clipboard::save_image(&src_dir.images(), &hash, &png, &img).unwrap();
        src.execute_batch(&format!(
            "INSERT INTO clips (hash, kind, text, html, source_app, pinned, shortcut, created_at, updated_at, use_count)
                 VALUES ('t', 'url', 'https://example.com', 'Version:0.9 x', 'chrome.exe', 1, 'Alt+Shift+M', 10, 20, 3);
             INSERT INTO clips (hash, kind, files, created_at, updated_at)
                 VALUES ('f', 'files', '[\"C:\\\\a.txt\"]', 11, 21);
             INSERT INTO clips (hash, kind, image_w, image_h, ocr_text, created_at, updated_at)
                 VALUES ('{hash}', 'image', 2, 3, 'seen', 12, 22);
             INSERT INTO snippets (name, body, shortcut, created_at, updated_at)
                 VALUES ('Sig', 'Regards, {{date}}', 'Ctrl+Alt+S', 1, 2);"
        ))
        .unwrap();
        let path = src_dir.0.join("out.wincy.json");
        let clips = export_clips(&src).unwrap();
        let snippets = db::snippets(&src).unwrap();
        let written = write_file(&path, clips, &snippets, Some(&src_dir.images())).unwrap();
        assert_eq!(
            written,
            ExportResult {
                clips: 3,
                snippets: 1
            }
        );

        let mut dst = db();
        let file = read_file(&path, MAX_FILE_BYTES).unwrap();
        let (result, ocr) = import_into(&mut dst, file, "Alt+V", true, &dst_dir.images()).unwrap();
        assert_eq!(
            result,
            ImportResult {
                added: 3,
                merged: 0,
                skipped: 0,
                snippets: 1
            }
        );
        assert!(ocr.is_empty(), "OCR text was imported");
        let clips = db::list(&dst, &dst_dir.images(), 2000).unwrap();
        let url = clips.iter().find(|c| c.kind == "url").unwrap();
        assert!(url.pinned && url.has_html);
        assert_eq!(url.shortcut.as_deref(), Some("Alt+Shift+M"));
        assert_eq!((url.created_at, url.updated_at, url.use_count), (10, 20, 3));
        assert_eq!(url.source_app.as_deref(), Some("chrome.exe"));
        let image = clips.iter().find(|c| c.kind == "image").unwrap();
        assert_eq!(image.image_size, Some((2, 3)));
        assert_eq!(image.ocr_text.as_deref(), Some("seen"));
        assert!(Path::new(image.image_path.as_ref().unwrap()).is_file());
        assert!(Path::new(image.thumb_path.as_ref().unwrap()).is_file());
        assert_eq!(
            clips.iter().find(|c| c.kind == "files").unwrap().files,
            Some(vec![r"C:\a.txt".to_string()])
        );
        let snippet = db::snippets(&dst).unwrap().remove(0);
        assert_eq!(
            (
                snippet.name.as_str(),
                snippet.body.as_str(),
                snippet.shortcut.as_deref()
            ),
            ("Sig", "Regards, {date}", Some("Ctrl+Alt+S"))
        );
        // Searchable through the FTS triggers.
        assert_eq!(
            db::search(&dst, &dst_dir.images(), "examp", 10, 2000)
                .unwrap()
                .len(),
            1
        );

        // Importing the same file again duplicates nothing.
        let file = read_file(&path, MAX_FILE_BYTES).unwrap();
        let (again, _) = import_into(&mut dst, file, "Alt+V", true, &dst_dir.images()).unwrap();
        assert_eq!(
            again,
            ImportResult {
                added: 0,
                merged: 3,
                skipped: 0,
                snippets: 0
            }
        );
        assert_eq!(db::count(&dst).unwrap(), 3);
        assert_eq!(db::snippets(&dst).unwrap().len(), 1);

        // Without images, the image clip is listed but cannot be restored.
        let clips = export_clips(&src).unwrap();
        write_file(&path, clips, &[], None).unwrap();
        let mut empty = db();
        let file = read_file(&path, MAX_FILE_BYTES).unwrap();
        let (r, _) = import_into(&mut empty, file, "Alt+V", true, &dst_dir.images()).unwrap();
        assert_eq!((r.added, r.skipped), (2, 1));
    }

    #[test]
    fn duplicates_merge_pins_and_shortcuts() {
        let dir = Dir::new("import-merge");
        let mut conn = db();
        let text = |t: &str, pinned: bool, shortcut: &str| {
            clip_json("text", &format!(r#","text":"{t}","shortcut":{shortcut}"#))
                .replace(r#""pinned":false"#, &format!(r#""pinned":{pinned}"#))
        };
        // Existing: "a" unpinned, "b" pinned with its own shortcut.
        let r = import(
            &mut conn,
            &envelope(
                &[
                    text("a", false, "null"),
                    text("b", true, r#""Alt+Shift+B""#),
                ],
                "",
            ),
            &dir,
        )
        .unwrap();
        assert_eq!(r.added, 2);
        let r = import(
            &mut conn,
            &envelope(
                &[
                    text("a", true, r#""Alt+Shift+A""#),
                    text("b", true, r#""Alt+Shift+X""#),
                    // In the file twice: the second one merges into the first.
                    text("c", false, "null"),
                    text("c", true, r#""Alt+V""#),
                ],
                "",
            ),
            &dir,
        )
        .unwrap();
        assert_eq!(
            r,
            ImportResult {
                added: 1,
                merged: 3,
                skipped: 0,
                snippets: 0
            }
        );
        let clips = db::list(&conn, &dir.images(), 2000).unwrap();
        let get = |t: &str| clips.iter().find(|c| c.preview == t).unwrap();
        assert!(get("a").pinned);
        assert_eq!(get("a").shortcut.as_deref(), Some("Alt+Shift+A"));
        assert_eq!(get("b").shortcut.as_deref(), Some("Alt+Shift+B"));
        // Pinned from the file, but the palette shortcut is not given away.
        assert!(get("c").pinned);
        assert_eq!(get("c").shortcut, None);
    }

    #[test]
    fn invalid_clips_are_skipped() {
        let dir = Dir::new("import-skip");
        let mut conn = db();
        let clips = [
            clip_json("text", r#","text":"ok""#),
            clip_json("text", r#","text":"   ""#),
            clip_json("text", ""),
            clip_json("weird", r#","text":"x""#),
            clip_json(
                "text",
                // Split so secret scanners don't flag this fake token.
                concat!(
                    r#","text":"gh"#,
                    r#"p_0123456789abcdefghijABCDEFGHIJ012345""#
                ),
            ),
            clip_json("files", r#","files":[]"#),
            clip_json("files", r#","files":["a\u0000b"]"#),
            clip_json("image", r#","imagePng":"not base64!""#),
            clip_json("image", r#","imagePng":"aGVsbG8=""#),
            clip_json("image", ""),
        ];
        let r = import(&mut conn, &envelope(&clips, ""), &dir).unwrap();
        assert_eq!(
            r,
            ImportResult {
                added: 1,
                merged: 0,
                skipped: 9,
                snippets: 0
            }
        );
        // Future timestamps are clamped, kinds recomputed, invalid snippets dropped.
        let r = import(
            &mut conn,
            &envelope(
                &[clip_json("text", r##","text":"#ff0000""##)
                    .replace(r#""updatedAt":2"#, r#""updatedAt":99999999999999"#)],
                r#"{"name":"","body":"x","shortcut":null,"createdAt":1,"updatedAt":1}"#,
            ),
            &dir,
        )
        .unwrap();
        assert_eq!((r.added, r.snippets), (1, 0));
        let color = db::list(&conn, &dir.images(), 10).unwrap().remove(0);
        assert_eq!(color.kind, "color");
        assert!(color.updated_at <= db::now_ms());
    }

    #[test]
    fn malformed_files_are_rejected() {
        let dir = Dir::new("import-bad");
        let mut conn = db();
        let ok = envelope(&[clip_json("text", r#","text":"x""#)], "");
        for bad in [
            String::new(),
            "not json".into(),
            "[]".into(),
            ok.replace("wincy-export", "other"),
            ok.replace(r#""version":1"#, r#""version":2"#),
            ok.replace(r#""exportedAt":5"#, r#""exportedAt":5,"extra":1"#),
            ok.replace(r#""useCount":0"#, r#""useCount":"zero""#),
            ok.replace(r#""useCount":0"#, r#""useCount":0,"evil":true"#),
            ok.replace(r#","snippets":[]"#, ""),
        ] {
            assert!(import(&mut conn, &bad, &dir).is_err(), "{bad}");
        }
        assert_eq!(db::count(&conn).unwrap(), 0);
        assert!(import(&mut conn, &ok, &dir).is_ok());
        // Errors never echo file contents.
        let e = import(
            &mut conn,
            &ok.replace(r#""text":"x""#, r#""text":12345"#),
            &dir,
        )
        .unwrap_err();
        assert!(!e.contains("12345"), "{e}");
    }

    #[test]
    fn oversize_files_and_bad_paths_are_rejected() {
        let dir = Dir::new("import-size");
        let path = dir.0.join("big.wincy.json");
        std::fs::write(&path, envelope(&[], "")).unwrap();
        assert!(read_file(&path, 1 << 20).is_ok());
        assert!(read_file(&path, 10).err().unwrap().contains("too large"));
        assert!(check_path("relative.wincy.json").is_err());
        assert!(check_path(r"C:\Windows\evil.dll").is_err());
        assert!(check_path(r"C:\x\backup.WINCY.JSON").is_ok());
    }
}
