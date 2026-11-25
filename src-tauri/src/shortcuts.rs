//! Global shortcuts of pinned clips and snippets. One plugin handler serves them and the
//! palette shortcut, dispatching by the pressed shortcut.

use std::collections::BTreeMap;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut};

use crate::db::{self, ShortcutTarget};
use crate::{AppState, paste, snippets, status, window};

/// Pin/snippet shortcuts currently registered, by `Shortcut::id()`.
static REGISTERED: Mutex<BTreeMap<u32, (Shortcut, ShortcutTarget)>> = Mutex::new(BTreeMap::new());
/// Serializes `sync` without holding `REGISTERED` while registering, which blocks on the
/// main thread, where `pressed` reads `REGISTERED`.
static SYNCING: Mutex<()> = Mutex::new(());
static PASTING: Mutex<()> = Mutex::new(());

const MAX_LEN: usize = 64;

/// Trims and parses a pin/snippet shortcut, rejecting the combos other apps rely on
/// (Ctrl(+Shift)+C/V/X/Z/Y/A) and the palette's Alt+1..9.
pub fn parse(text: &str) -> Result<(String, Shortcut), String> {
    let text = text.trim();
    let shortcut = (text.len() <= MAX_LEN)
        .then(|| text.parse::<Shortcut>().ok())
        .flatten()
        .ok_or_else(|| "Invalid shortcut".to_string())?;
    let reserved = "CVXZYA"
        .chars()
        .flat_map(|k| [format!("Ctrl+{k}"), format!("Ctrl+Shift+{k}")])
        .chain((1..=9).map(|d| format!("Alt+{d}")))
        .any(|r| same(&r, &shortcut));
    if reserved {
        return Err(format!("{text} is reserved"));
    }
    // Without Ctrl/Alt/Win a global shortcut would swallow normal typing system-wide.
    let fkey = shortcut
        .key
        .to_string()
        .strip_prefix('F')
        .is_some_and(|n| n.parse::<u8>().is_ok());
    if !fkey
        && !shortcut
            .mods
            .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER)
    {
        return Err(format!("{text} needs Ctrl, Alt or Win"));
    }
    Ok((text.to_string(), shortcut))
}

fn same(text: &str, shortcut: &Shortcut) -> bool {
    text.parse::<Shortcut>()
        .is_ok_and(|s| s.id() == shortcut.id())
}

/// True when the palette shortcut `main`, a pin or a snippet other than `except` uses it.
pub fn taken(
    conn: &Connection,
    main: &str,
    shortcut: &Shortcut,
    except: Option<ShortcutTarget>,
) -> rusqlite::Result<bool> {
    Ok(same(main, shortcut)
        || db::shortcuts(conn)?
            .iter()
            .any(|(text, target)| Some(*target) != except && same(text, shortcut)))
}

/// `parse` plus uniqueness across the palette, pins and snippets. Returns the trimmed text.
pub fn validate(
    conn: &Connection,
    main: &str,
    text: &str,
    except: Option<ShortcutTarget>,
) -> Result<String, String> {
    let (text, shortcut) = parse(text)?;
    if taken(conn, main, &shortcut, except).map_err(|e| e.to_string())? {
        return Err(format!("{text} is already used by Wincy"));
    }
    Ok(text)
}

/// Plugin handler entry point (key down).
pub fn pressed(app: &AppHandle, shortcut: &Shortcut) {
    let target = REGISTERED
        .lock()
        .unwrap()
        .get(&shortcut.id())
        .map(|(_, t)| *t);
    let Some(target) = target else {
        window::toggle(app);
        return;
    };
    // RegisterHotKey hides the combo from the WebView, so a recorder there never sees it, and
    // the paste would go to the app used before Wincy opened. Ignore it and tell the recorder.
    if window::own_window_is_foreground(app) {
        let _ = app.emit("shortcut-in-use", ());
        return;
    }
    // The palette is not shown: the paste goes to the window focused now.
    window::remember_target(app);
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // One paste at a time: each writes the clipboard, then waits for the modifiers before
        // Ctrl+V, so overlapping pastes would send the last clipboard content twice.
        let _one = PASTING.lock().unwrap_or_else(|e| e.into_inner());
        let result = match target {
            ShortcutTarget::Clip(id) => paste::paste(&app, id, false),
            ShortcutTarget::Snippet(id) => snippets::paste(&app, id),
        };
        if let Err(e) = result {
            log::warn!("shortcut paste failed: {e}");
        }
    });
}

/// Registers every stored pin/snippet shortcut, unregisters stale ones and reports the
/// ones that could not be registered in `Status.shortcutConflicts`.
/// Not on the main thread once the app runs: registering waits for it.
pub fn sync(app: &AppHandle) {
    let _syncing = SYNCING.lock().unwrap();
    let state = app.state::<AppState>();
    let wanted = {
        let conn = state.db.lock().unwrap();
        db::shortcuts(&conn)
    };
    let wanted = match wanted {
        Ok(w) => w,
        Err(e) => {
            log::warn!("shortcuts: {e}");
            return;
        }
    };
    let main = state.settings.lock().unwrap().shortcut.clone();
    let gs = app.global_shortcut();
    let mut old = REGISTERED.lock().unwrap().clone();
    let mut next = BTreeMap::new();
    let mut conflicts = Vec::new();
    for (text, target) in wanted {
        let kind = match target {
            ShortcutTarget::Clip(_) => "pin",
            ShortcutTarget::Snippet(_) => "snippet",
        };
        // `parse`, not `str::parse`: rows that predate a rule (or were imported) stay unregistered.
        let ok = match parse(&text).map(|(_, sc)| sc) {
            Ok(sc) if !same(&main, &sc) && !next.contains_key(&sc.id()) => {
                let live = old.remove(&sc.id()).is_some()
                    || gs
                        .register(sc)
                        .inspect_err(|e| log::warn!("shortcut {text} ({kind}): {e}"))
                        .is_ok();
                if live {
                    next.insert(sc.id(), (sc, target));
                }
                live
            }
            _ => false,
        };
        if !ok {
            conflicts.push(format!("{text} ({kind})"));
        }
    }
    for (sc, _) in old.values() {
        let _ = gs.unregister(*sc);
    }
    *REGISTERED.lock().unwrap() = next;
    status::update(app, |s| s.shortcut_conflicts = conflicts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_reserved_and_invalid() {
        assert_eq!(parse(" Alt+Shift+M ").unwrap().0, "Alt+Shift+M");
        assert!(parse("Ctrl+Alt+1").is_ok());
        for bad in [
            "Ctrl+C",
            "ctrl+shift+v",
            "Control+A",
            "Alt+1",
            "alt+9",
            "",
            "Alt+Nope",
            "Enter",
            "Space",
            "A",
            "Shift+A",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        assert!(parse("F9").is_ok());
        assert!(parse("Shift+F9").is_ok());
        assert!(parse(&format!("Alt+{}", "x".repeat(70))).is_err());
    }

    #[test]
    fn taken_is_case_and_order_insensitive() {
        let conn = Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO clips (hash, kind, text, pinned, created_at, updated_at, shortcut)
                 VALUES ('a', 'text', 'x', 1, 1, 1, 'Alt+Shift+M');
             INSERT INTO snippets (name, body, shortcut, created_at, updated_at)
                 VALUES ('s', 'b', 'Ctrl+Alt+S', 1, 1);",
        )
        .unwrap();
        let sc = |t: &str| t.parse::<Shortcut>().unwrap();
        assert!(taken(&conn, "Alt+V", &sc("shift+alt+m"), None).unwrap());
        assert!(
            !taken(
                &conn,
                "Alt+V",
                &sc("Alt+Shift+M"),
                Some(ShortcutTarget::Clip(1))
            )
            .unwrap()
        );
        assert!(taken(&conn, "Alt+V", &sc("alt+v"), None).unwrap());
        assert!(validate(&conn, "Alt+V", "Ctrl+Alt+S", Some(ShortcutTarget::Clip(1))).is_err());
        assert_eq!(
            validate(
                &conn,
                "Alt+V",
                "Ctrl+Alt+S",
                Some(ShortcutTarget::Snippet(1))
            )
            .unwrap(),
            "Ctrl+Alt+S"
        );
    }
}
