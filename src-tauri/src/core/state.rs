// wallpaper state — persistence for active wallpapers across app restarts
//
// saves which wallpaper is set on which monitor so the app can
// restore them on next launch.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    pub static ref WALLPAPER_STATE: Mutex<WallpaperState> = Mutex::new(WallpaperState::default());
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WallpaperState {
    pub is_active: bool,
    pub monitor_wallpapers: HashMap<String, MonitorWallpaper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorWallpaper {
    pub kind: WallpaperKind,
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WallpaperKind {
    Video,
    Scene,
    Interactive,
}

/// load wallpaper state from disk
pub fn load_state() -> WallpaperState {
    let state_path = get_state_path();
    match std::fs::read_to_string(&state_path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => WallpaperState::default(),
    }
}

/// save wallpaper state to disk
pub fn save_state(state: &WallpaperState) {
    let state_path = get_state_path();
    if let Some(parent) = state_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(&state_path, json);
    }
}

fn get_state_path() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ColorWall")
        .join("wallpaper-state.json")
}
