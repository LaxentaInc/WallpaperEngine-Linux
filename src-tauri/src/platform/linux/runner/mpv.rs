// video::mpv - libmpv video playback engine
//
// pure mpv initialization and control. this module is completely
// shell-agnostic: it receives a window id from whatever DesktopSurface
// implementation created the background window, and renders into it.
//
// on linux, mpv supports multiple hardware decode backends:
// - vaapi (intel, amd) - most common on linux
// - nvdec (nvidia proprietary driver)
// - vulkan (via gpu-next output driver)
//
// mpv auto-detects the best available backend when hwdec=auto.
//
// this is the linux equivalent of:
// - platform/windows/mpv/ (mpv backend on windows)
// - platform/windows/wmf/ (windows media foundation backend)

/// configuration for initializing the mpv player.
/// the sidecar entry point (cl_vp.rs) builds this from CLI args
/// and passes it to initialize().
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

/// initialize libmpv and start playing the video.
///
/// this function:
/// 1. creates an mpv context
/// 2. sets hardware decode to auto (vaapi/nvdec/software fallback)
/// 3. sets the render target to the given window id
/// 4. loads the video file and starts playback
/// 5. blocks until mpv exits or receives a stop command
pub fn initialize(config: &MpvConfig) -> Result<(), String> {
    // todo: implement using the mpv crate (libmpv rust bindings)
    //
    // pseudocode:
    // let mpv = mpv::Mpv::new()?;
    // mpv.set_property("wid", config.window_id)?;
    // mpv.set_property("hwdec", "auto")?;
    // mpv.set_property("vo", "gpu-next")?;    // modern vulkan output
    // mpv.set_property("loop-file", "inf")?;
    // mpv.set_property("volume", config.volume)?;
    // mpv.command("loadfile", &[&config.video_path])?;
    // mpv.event_loop();  // blocks until quit

    println!(
        "[mpv] would initialize with video='{}' window_id={} loop={} volume={}",
        config.video_path, config.window_id, config.loop_playback, config.volume
    );
    Err("mpv initialization not implemented yet".to_string())
}
