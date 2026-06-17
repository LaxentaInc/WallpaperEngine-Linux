// data models — shared types for wallpapers, settings, etc.
//
// todo: bring over AppSettings, WallpaperItem, etc. from the windows version

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub video_player: String,        // "mpv" or "gstreamer"
    pub audio_enabled: bool,
    pub live_wallpaper_enabled: bool,
    pub pause_on_fullscreen: bool,
    pub hwdec: String,               // "auto", "vaapi", "nvdec", "none"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            video_player: "mpv".to_string(),
            audio_enabled: false,
            live_wallpaper_enabled: true,
            pause_on_fullscreen: true,
            hwdec: "auto".to_string(),
        }
    }
}
