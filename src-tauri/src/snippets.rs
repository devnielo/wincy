//! Snippets: named text templates, pasted as plain text with their variables expanded.

use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Globalization::{
    DATE_SHORTDATE, GetDateFormatEx, GetTimeFormatEx, TIME_FORMAT_FLAGS,
};
use windows::core::{PCWSTR, w};

use crate::db::{self, ShortcutTarget, Snippet};
use crate::{AppState, clipboard, paste, shortcuts};

pub const MAX_NAME: usize = 80;
pub const MAX_BODY: usize = 100_000;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetInput {
    pub id: Option<i64>,
    pub name: String,
    pub body: String,
    pub shortcut: Option<String>,
}

/// Returns the trimmed name. Lengths are in characters.
pub fn validate(name: &str, body: &str) -> Result<String, String> {
    let name = name.trim();
    if !(1..=MAX_NAME).contains(&name.chars().count()) {
        return Err(format!("The name must have 1 to {MAX_NAME} characters"));
    }
    if !(1..=MAX_BODY).contains(&body.chars().count()) {
        return Err(format!("The text must have 1 to {MAX_BODY} characters"));
    }
    Ok(name.to_string())
}

fn changed(app: &AppHandle) {
    let _ = app.emit("snippets-changed", ());
}

pub fn save(app: &AppHandle, input: SnippetInput) -> Result<Snippet, String> {
    let name = validate(&input.name, &input.body)?;
    let state = app.state::<AppState>();
    let main = state.settings.lock().unwrap().shortcut.clone();
    let saved = {
        let conn = state.db.lock().unwrap();
        let shortcut = input
            .shortcut
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| shortcuts::validate(&conn, &main, s, input.id.map(ShortcutTarget::Snippet)))
            .transpose()?;
        db::save_snippet(&conn, input.id, &name, &input.body, shortcut.as_deref())
            .map_err(|e| e.to_string())?
            .ok_or("Snippet not found")?
    };
    shortcuts::sync(app);
    changed(app);
    Ok(saved)
}

pub fn delete(app: &AppHandle, id: i64) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        let conn = state.db.lock().unwrap();
        db::delete_snippet(&conn, id).map_err(|e| e.to_string())?;
    }
    shortcuts::sync(app);
    changed(app);
    Ok(())
}

/// Blocks (paste flow); not on the main thread.
pub fn paste(app: &AppHandle, id: i64) -> Result<(), String> {
    let body = {
        let state = app.state::<AppState>();
        let conn = state.db.lock().unwrap();
        db::snippet(&conn, id).map_err(|e| e.to_string())?
    }
    .ok_or("Snippet not found")?
    .body;
    let (date, time) = (local_date(), local_time());
    let mut clipboard = None;
    let text = expand(&body, |name| match name {
        "date" => Some(date.clone()),
        "time" => Some(time.clone()),
        "datetime" => Some(format!("{date} {time}")),
        "clipboard" => Some(
            clipboard
                .get_or_insert_with(|| clipboard::current_text().unwrap_or_default())
                .clone(),
        ),
        _ => None,
    });
    paste::paste_text(app, &text)
}

/// Replaces `{name}` with `var(name)`; `{{` is a literal `{`; unknown names stay as written.
pub fn expand(body: &str, mut var: impl FnMut(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if let Some(after) = rest.strip_prefix("{{") {
            out.push('{');
            rest = after;
            continue;
        }
        if let Some(end) = rest.find('}')
            && let Some(value) = var(&rest[1..end])
        {
            out.push_str(&value);
            rest = &rest[end + 1..];
            continue;
        }
        out.push('{');
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}

fn win_string(f: impl FnOnce(&mut [u16]) -> i32) -> String {
    let mut buf = [0u16; 128];
    // The count includes the terminating NUL; 0 means failure.
    let len = f(&mut buf).max(1) as usize;
    String::from_utf16_lossy(&buf[..len - 1])
}

/// Today in the user's short date format.
fn local_date() -> String {
    win_string(|buf| unsafe {
        GetDateFormatEx(
            PCWSTR::null(),
            DATE_SHORTDATE,
            None,
            PCWSTR::null(),
            Some(buf),
            PCWSTR::null(),
        )
    })
}

/// Local time as HH:MM (24 h).
fn local_time() -> String {
    win_string(|buf| unsafe {
        GetTimeFormatEx(
            PCWSTR::null(),
            TIME_FORMAT_FLAGS(0),
            None,
            w!("HH:mm"),
            Some(buf),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(name: &str) -> Option<String> {
        match name {
            "date" => Some("25/11/2026".into()),
            "clipboard" => Some("{date}".into()),
            _ => None,
        }
    }

    #[test]
    fn expands_known_variables() {
        assert_eq!(expand("Hi {date}!", vars), "Hi 25/11/2026!");
        // Values are not expanded again.
        assert_eq!(expand("{clipboard}", vars), "{date}");
        assert_eq!(expand("", vars), "");
        assert_eq!(expand("ñ{date}ñ", vars), "ñ25/11/2026ñ");
    }

    #[test]
    fn escapes_and_unknown_names() {
        assert_eq!(expand("{{date}", vars), "{date}");
        assert_eq!(expand("{{{date}", vars), "{25/11/2026");
        assert_eq!(expand("{name} {}", vars), "{name} {}");
        assert_eq!(expand("fn x() { {date} }", vars), "fn x() { 25/11/2026 }");
        assert_eq!(expand("open { never closed", vars), "open { never closed");
        assert_eq!(expand("{a{date}", vars), "{a25/11/2026");
        assert_eq!(expand("trailing {", vars), "trailing {");
    }

    #[test]
    fn local_date_and_time_are_formatted() {
        assert!(!local_date().is_empty());
        let t = local_time();
        assert_eq!(t.len(), 5, "{t}");
        assert_eq!(&t[2..3], ":");
    }

    #[test]
    fn validation() {
        assert_eq!(validate("  Sig  ", "body").unwrap(), "Sig");
        assert!(validate(" ", "body").is_err());
        assert!(validate(&"é".repeat(80), "b").is_ok());
        assert!(validate(&"é".repeat(81), "b").is_err());
        assert!(validate("n", "").is_err());
        assert!(validate("n", &"x".repeat(MAX_BODY + 1)).is_err());
    }
}
