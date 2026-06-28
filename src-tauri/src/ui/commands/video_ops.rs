// ui::commands::video_ops - wallpaper control commands
//
// these are the tauri #[tauri::command] functions that the react
// frontend calls when the user clicks "set wallpaper" or "stop".
// they delegate to core::engine_video::process which handles
// the actual sidecar spawning.

use crate::core::engine_video;

/// set a video wallpaper on a monitor (or primary if none specified).
/// called from the frontend via tauri's invoke system.
#[tauri::command]
pub fn cmd_set_video_wallpaper(
    app: tauri::AppHandle,
    video_path: String,
    monitor_id: Option<String>,
) -> Result<(), String> {
    engine_video::set_video_wallpaper(&app, &video_path, monitor_id.as_deref())
}

/// stop all active wallpapers across all monitors.
#[tauri::command]
pub fn cmd_stop_wallpaper() -> Result<(), String> {
    engine_video::stop_all()
}
