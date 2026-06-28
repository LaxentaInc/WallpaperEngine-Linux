// types - shared data structures used across all compositor implementations
//
// these types are the common language between detection, surface creation,
// and the video player. every shell folder maps its native monitor
// representation into these structs.

use serde::{Deserialize, Serialize};

/// information about a physical display/monitor.
/// each compositor has its own way of enumerating monitors
/// (wl_output on wayland, XRandR on x11) but they all map
/// their native data into this struct so the rest of the app
/// can work with a unified representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    /// unique identifier for this monitor (compositor-specific format)
    pub id: String,
    /// human-readable name (e.g. "DP-1", "HDMI-A-1", "eDP-1")
    pub name: String,
    /// x position in the virtual coordinate space (pixels)
    pub x: i32,
    /// y position in the virtual coordinate space (pixels)
    pub y: i32,
    /// width in pixels
    pub width: u32,
    /// height in pixels
    pub height: u32,
    /// scale factor (e.g. 1.0 for 1080p, 2.0 for hidpi)
    pub scale: f64,
    /// whether this is the user's primary monitor
    pub primary: bool,
}

/// the detected compositor capability that determines which
/// code path the video player will take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellCapability {
    /// compositor supports wlr-layer-shell protocol.
    /// covers: hyprland, sway, river, wayfire, labwc, niri, kde plasma, cosmic.
    LayerShell { compositor_name: String },
    /// running on x11 (any window manager).
    /// covers: i3, openbox, xfce, mate, cinnamon, etc.
    X11 { desktop: String },
    /// gnome/mutter on wayland. does not support layer-shell.
    /// needs a completely different approach (xwayland fallback).
    Mutter,
    /// could not detect. should not happen on a graphical system.
    Unknown,
}
