//! Paste flow: clipboard write, focus restoration, simulated Ctrl+V.

use std::time::{Duration, Instant};

use enigo::{Direction, Enigo, Key, Keyboard};
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

use crate::{AppState, clipboard, db, window};

/// Virtual-key code of V: layout independent, unlike `Key::Unicode('v')`.
const VK_V: u32 = 0x56;

pub fn paste(app: &AppHandle, id: i64, plain: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    let clip = {
        let conn = state.db.lock().unwrap();
        db::get(&conn, id, &state.images_dir).map_err(|e| e.to_string())?
    }
    .ok_or("Clip not found")?;

    clipboard::write(&clip, plain)?;
    {
        let conn = state.db.lock().unwrap();
        db::mark_used(&conn, id).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("clips-changed", ());

    window::hide_and_restore(app);
    if !state.settings.lock().unwrap().paste_directly {
        return Ok(());
    }
    // A still-held Alt (Alt+1..9) or Shift would turn Ctrl+V into another shortcut. If they
    // stay down, skip it: the clip is already on the clipboard for a manual Ctrl+V.
    if !wait_for_modifiers_release(Duration::from_millis(600)) {
        return Ok(());
    }
    std::thread::sleep(Duration::from_millis(40));
    send_ctrl_v()
}

/// Returns false if a modifier is still held after `timeout`.
fn wait_for_modifiers_release(timeout: Duration) -> bool {
    let start = Instant::now();
    let held = || {
        [VK_MENU, VK_SHIFT, VK_CONTROL, VK_LWIN, VK_RWIN]
            .iter()
            .any(|vk| unsafe { GetAsyncKeyState(vk.0 as i32) } < 0)
    };
    while held() && start.elapsed() < timeout {
        std::thread::sleep(Duration::from_millis(10));
    }
    !held()
}

fn send_ctrl_v() -> Result<(), String> {
    let mut enigo = Enigo::new(&enigo::Settings::default()).map_err(|e| e.to_string())?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| e.to_string())?;
    let result = enigo.key(Key::Other(VK_V), Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);
    result.map_err(|e| e.to_string())
}
