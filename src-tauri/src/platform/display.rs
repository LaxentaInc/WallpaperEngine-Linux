// display server detection and abstraction
//
// linux has multiple display server protocols that each need
// completely different approaches to create a background window:
//
// - wayland (wlroots): zwlr_layer_shell_v1 — clean background layer
// - wayland (gnome):   xwayland fallback with ewmh hints
// - x11:               _NET_WM_WINDOW_TYPE_DESKTOP + _NET_WM_STATE_BELOW
//
// the sidecar binaries (cl-video-player, cl-scene-renderer) use this
// detection to decide which code path to take at startup.

use serde::{Deserialize, Serialize};

/// the detected display server environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayServer {
    /// wlroots-based compositor (hyprland, sway, etc.) — best support
    WaylandWlroots { compositor: String },
    /// kde plasma on wayland — supports layer-shell
    WaylandKDE,
    /// gnome on wayland — no layer-shell, needs xwayland fallback
    WaylandGnome,
    /// generic/unknown wayland compositor
    WaylandUnknown { desktop: String },
    /// x11 with a standard window manager
    X11 { desktop: String },
    /// could not detect (shouldn't happen on a graphical linux system)
    Unknown,
}

/// known wlroots-based compositors that support zwlr_layer_shell_v1
const WLROOTS_COMPOSITORS: &[&str] = &[
    "hyprland",
    "sway",
    "wayfire",
    "river",
    "labwc",
    "cage",
];

/// detect the active display server by reading environment variables.
/// this runs once at app startup and the result is passed to sidecar binaries.
pub fn detect() -> DisplayServer {
    let session_type = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_lowercase();

    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default();

    let desktop_lower = desktop.to_lowercase();

    match session_type.as_str() {
        "wayland" => {
            // check for wlroots-based compositors (best support)
            for &compositor in WLROOTS_COMPOSITORS {
                if desktop_lower.contains(compositor) {
                    println!("[display] detected wlroots compositor: {}", desktop);
                    return DisplayServer::WaylandWlroots {
                        compositor: compositor.to_string(),
                    };
                }
            }

            // check hyprland specifically via its own env var
            if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
                println!("[display] detected hyprland via instance signature");
                return DisplayServer::WaylandWlroots {
                    compositor: "hyprland".to_string(),
                };
            }

            // kde plasma
            if desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
                println!("[display] detected kde plasma wayland");
                return DisplayServer::WaylandKDE;
            }

            // gnome — the hard one
            if desktop_lower.contains("gnome") || desktop_lower.contains("ubuntu") {
                println!("[display] detected gnome wayland (will use xwayland fallback)");
                return DisplayServer::WaylandGnome;
            }

            println!("[display] unknown wayland compositor: {}", desktop);
            DisplayServer::WaylandUnknown { desktop }
        }
        "x11" | "tty" => {
            println!("[display] detected x11 session: {}", desktop);
            DisplayServer::X11 { desktop }
        }
        _ => {
            // fallback: check if DISPLAY is set (x11) or WAYLAND_DISPLAY (wayland)
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                println!("[display] detected wayland via WAYLAND_DISPLAY env");
                return DisplayServer::WaylandUnknown { desktop };
            }
            if std::env::var("DISPLAY").is_ok() {
                println!("[display] detected x11 via DISPLAY env");
                return DisplayServer::X11 { desktop };
            }

            println!("[display] could not detect display server");
            DisplayServer::Unknown
        }
    }
}

impl DisplayServer {
    /// whether this display server supports the wlr-layer-shell protocol
    /// (allows clean background-layer window placement)
    pub fn supports_layer_shell(&self) -> bool {
        matches!(
            self,
            DisplayServer::WaylandWlroots { .. } | DisplayServer::WaylandKDE
        )
    }

    /// whether we need the xwayland/x11 ewmh fallback path
    pub fn needs_x11_fallback(&self) -> bool {
        matches!(
            self,
            DisplayServer::WaylandGnome | DisplayServer::X11 { .. }
        )
    }

    /// short name for logging and ipc
    pub fn name(&self) -> &str {
        match self {
            DisplayServer::WaylandWlroots { .. } => "wayland-wlroots",
            DisplayServer::WaylandKDE => "wayland-kde",
            DisplayServer::WaylandGnome => "wayland-gnome",
            DisplayServer::WaylandUnknown { .. } => "wayland-unknown",
            DisplayServer::X11 { .. } => "x11",
            DisplayServer::Unknown => "unknown",
        }
    }
}
