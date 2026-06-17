// platform — linux display server abstraction and wallpaper engine
//
// this module handles the core difference between linux and windows:
// instead of DWM + Win32 APIs, we deal with wayland compositors,
// x11 window managers, and their wildly different protocols.

pub mod display;
pub mod engine;
pub mod ipc;

use display::DisplayServer;

/// detect which display server is active
pub fn detect_display_server() -> DisplayServer {
    display::detect()
}

// --- tauri commands ---

/// set a video wallpaper on a monitor (or primary if none specified)
#[tauri::command]
pub fn cmd_set_video_wallpaper(
    app: tauri::AppHandle,
    video_path: String,
    monitor_id: Option<String>,
) -> Result<(), String> {
    engine::set_video_wallpaper(&app, &video_path, monitor_id.as_deref())
}

/// stop all wallpapers
#[tauri::command]
pub fn cmd_stop_wallpaper() -> Result<(), String> {
    engine::stop_wallpaper()
}

/// get display server info + available monitors
#[tauri::command]
pub fn cmd_get_display_info() -> Result<serde_json::Value, String> {
    let server = detect_display_server();
    Ok(serde_json::json!({
        "display_server": format!("{:?}", server),
        "session_type": std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
        "desktop": std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
    }))
}
