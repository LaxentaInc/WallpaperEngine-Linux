// platform::linux::x11 - X11 window manager support
//
// this folder handles ALL x11 sessions, regardless of window manager.
// x11 uses EWMH (Extended Window Manager Hints) which is universally
// supported by all x11 WMs: i3, openbox, xfce, mate, cinnamon, etc.
//
// the approach:
// 1. create a window with winit
// 2. set _NET_WM_WINDOW_TYPE to _NET_WM_WINDOW_TYPE_DESKTOP
// 3. set _NET_WM_STATE to include _NET_WM_STATE_BELOW
// 4. this tells the WM to treat it as the desktop background

pub mod surface;
pub mod monitors;
