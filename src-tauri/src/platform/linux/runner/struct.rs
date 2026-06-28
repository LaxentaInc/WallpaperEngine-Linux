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