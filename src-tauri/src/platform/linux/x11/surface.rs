// x11::surface - create a desktop-background X11 window
//
// uses winit to create a borderless, fullscreen window, then sets
// X11 EWMH properties to force the window manager to treat it as
// the desktop background layer.
//
// properties set:
//   _NET_WM_WINDOW_TYPE = _NET_WM_WINDOW_TYPE_DESKTOP
//   _NET_WM_STATE       = _NET_WM_STATE_BELOW, _NET_WM_STATE_STICKY
//
// this is the x11 equivalent of the WorkerW/Progman hack on windows,
// but using the official EWMH spec instead of undocumented win32 tricks.

use crate::platform::linux::shared::surface_trait::DesktopSurface;
use crate::platform::linux::shared::types::MonitorInfo;

pub struct X11Surface {
    // todo: store the winit window and x11 connection
    _monitor: MonitorInfo,
}

impl DesktopSurface for X11Surface {
    fn create(monitor: &MonitorInfo) -> Result<Self, String> {
        // todo: implement using winit + x11rb or raw xlib
        // 1. create an event loop with winit
        // 2. build a borderless, undecorated window at monitor coordinates
        // 3. get the raw x11 window id from winit
        // 4. use x11rb to set _NET_WM_WINDOW_TYPE_DESKTOP
        // 5. set _NET_WM_STATE_BELOW to force it behind everything
        // 6. map the window
        println!(
            "[x11] creating desktop window on monitor '{}'",
            monitor.name
        );
        Err("x11 surface not implemented yet".to_string())
    }

    fn window_id(&self) -> u64 {
        // todo: return the x11 window id (XID) for mpv's --wid flag
        0
    }

    fn run_event_loop(&mut self) -> Result<(), String> {
        // todo: pump x11 events via winit's event loop
        Err("event loop not implemented yet".to_string())
    }

    fn destroy(&mut self) {
        println!("[x11] destroying desktop window");
    }
}
