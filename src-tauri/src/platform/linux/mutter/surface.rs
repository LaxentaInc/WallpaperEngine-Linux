// mutter::surface - gnome desktop background surface (xwayland fallback)
//
// since mutter refuses wlr-layer-shell, we fall back to creating an
// x11 window under gnome's xwayland compatibility layer. the window
// gets EWMH _NET_WM_WINDOW_TYPE_DESKTOP hints just like the x11/ folder,
// but runs inside gnome's xwayland server instead of native x11.
//
// limitations:
// - slightly higher latency than native wayland (extra compositor pass)
// - may not work if gnome disables xwayland in future releases
// - fractional scaling may be inconsistent

use crate::platform::linux::shared::surface_trait::DesktopSurface;
use crate::platform::linux::shared::types::MonitorInfo;

pub struct MutterSurface {
    // todo: store the xwayland window state
    // this will likely reuse x11 surface code internally
    _monitor: MonitorInfo,
}

impl DesktopSurface for MutterSurface {
    fn create(monitor: &MonitorInfo) -> Result<Self, String> {
        // todo: implement xwayland fallback
        // largely the same as x11::surface but connecting to
        // gnome's xwayland display instead of a native x11 server
        println!(
            "[mutter] creating xwayland desktop surface on monitor '{}'",
            monitor.name
        );
        Err("mutter surface not implemented yet".to_string())
    }

    fn window_id(&self) -> u64 {
        0
    }

    fn run_event_loop(&mut self) -> Result<(), String> {
        Err("event loop not implemented yet".to_string())
    }

    fn destroy(&mut self) {
        println!("[mutter] destroying xwayland surface");
    }
}
