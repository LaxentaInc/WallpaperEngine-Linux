// surface_trait - the contract every compositor implementation must fulfill
//
// this trait is the core abstraction that makes cl_vp.rs compositor-agnostic.
// layer_shell, x11, and mutter each implement this trait differently,
// but cl_vp.rs only ever talks to this interface.

use super::types::MonitorInfo;

/// every compositor must implement this to create a wallpaper-layer window.
/// the video player (mpv.rs) receives a DesktopSurface and renders onto it
/// without knowing which compositor created it.
pub trait DesktopSurface {
    /// create a new background-layer window on the specified monitor.
    /// on layer_shell: this requests LAYER_BACKGROUND via wlr-layer-shell.
    /// on x11: this sets _NET_WM_WINDOW_TYPE_DESKTOP.
    /// on mutter: this uses the xwayland fallback approach.
    fn create(monitor: &MonitorInfo) -> Result<Self, String>
    where
        Self: Sized;

    /// returns the raw window handle that mpv will render into.
    /// this is the critical bridge: mpv's render context (wid wont work for us)
    /// needs this platform-specific handle to paint frames.
    fn window_id(&self) -> u64;

    /// run the event loop for this surface.
    /// on wayland this pumps wl_display events.
    /// on x11 this pumps the x event queue.
    /// this blocks the current thread.
    fn run_event_loop(&mut self) -> Result<(), String>;

    /// cleanly destroy the surface and release compositor resources.
    fn destroy(&mut self);
}
