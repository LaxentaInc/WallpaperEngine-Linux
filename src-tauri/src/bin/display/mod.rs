use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

pub mod wayland;
pub mod x11;

/// the DesktopSurface trait defines the contract for any display server wrapper.
/// whether we are on X11 or Wayland, the video engine just needs a raw window handle
/// that is guaranteed to be pinned to the desktop background.
pub trait DesktopSurface: HasRawWindowHandle + HasRawDisplayHandle {
    /// gets the underlying winit window (if needed for event pumping)
    fn window(&self) -> &Window;

    /// force the window to stay at the absolute bottom z-index (behind icons)
    fn set_background_layer(&self);
}

/// factory function to spawn the correct surface based on the environment
pub fn create_surface(display_server: &str) -> Result<Box<dyn DesktopSurface>, String> {
    match display_server {
        "wayland-wlroots" | "wayland-kde" => {
            // TODO: initialize wayland::WaylandSurface
            Err("wayland surface not implemented yet".to_string())
        }
        "x11" => {
            // TODO: initialize x11::X11Surface
            Err("x11 surface not implemented yet".to_string())
        }
        _ => Err(format!("unsupported display server: {}", display_server)),
    }
}
