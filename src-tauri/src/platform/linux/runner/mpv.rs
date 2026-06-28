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

use super::config::MpvConfig;
use libmpv2::Mpv;

/// initialize libmpv and start playing the video.
pub fn initialize(config: &MpvConfig) -> Result<(), String> {
    println!("[mpv] initializing libmpv context...");
    let mpv = Mpv::new().map_err(|e| format!("failed to create mpv context: {}", e))?;
    // pass the raw window id so mpv renders into our background surface.
    // if window_id is 0, mpv will spawn its own test window (great for debugging!)
    if config.window_id != 0 {
        mpv.set_property("wid", config.window_id as i64)
            .map_err(|e| format!("failed to set window id: {}", e))?;
    }

    mpv.set_property("hwdec", "auto-safe").unwrap(); // uses vaapi/nvdec safely
    mpv.set_property("vo", "gpu").unwrap();
    
    mpv.set_property("profile", "fast").unwrap();
    mpv.set_property("vd-lavc-fast", "yes").unwrap();
    mpv.set_property("vd-lavc-skiploopfilter", "all").unwrap(); // huge CPU save for h264
    mpv.set_property("osc", "no").unwrap();
    mpv.set_property("window-dragging", "no").unwrap();
    mpv.set_property("input-default-bindings", "no").unwrap();
    mpv.set_property("audio", "no").unwrap();
    mpv.set_property("border", "no").unwrap();

    // apply wallpaper properties
    if config.loop_playback {
        mpv.set_property("loop-file", "inf").unwrap();
    }
    mpv.set_property("volume", config.volume as i64).unwrap();
    println!("[mpv] loading video: {}", config.video_path);
    mpv.command("loadfile", &[&config.video_path])
        .map_err(|e| format!("failed to load video: {}", e))?;

    // events to keep the player alive
    // (in the future, we will handle IPC commands here to pause/resume)
    loop {
        if let Some(_event) = mpv.wait_event(1.0) {
            // process event if needed
        }
    }
}