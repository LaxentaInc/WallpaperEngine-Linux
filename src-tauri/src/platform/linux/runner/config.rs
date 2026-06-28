pub struct MpvConfig {
    /// path to the video file to play
    pub video_path: String,
    /// the raw window id to render into (from DesktopSurface::window_id())
    pub window_id: u64,
    /// whether to loop the video infinitely (wallpaper mode)
    pub loop_playback: bool,
    /// volume level (0-100, typically 0 for wallpapers)
    pub volume: u32,
}

impl Default for MpvConfig { 
    fn default() -> Self { 

        Self {
            video_path: String::new(),
            window_id: 0,
            loop_playback: true,
            volume: 0,
        }
    }

}

// try to evolve forward to our windows one (We prob wont rlly need mpv path here but yes) 
// pub fn run_mpv_player(
    // video_path: &str,
    // mpv_path: Option<String>,
    // audio_enabled: bool,
    // paused: bool,
    // pause_on_fullscreen: bool,
    // target: &PlayerTarget,
    // mpv_preset: &str,
    // debug_mode: bool,