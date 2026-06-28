// detection - compositor capability detection
//
// reads linux environment variables to figure out which compositor
// is running and what protocols it supports. the result (ShellCapability)
// tells cl_vp.rs which folder's implementation to use.
//
// this runs once at startup. the main tauri app calls it to log the
// environment, and also passes the result as a CLI arg to the sidecar.

use super::types::ShellCapability;

/// known compositors that implement the wlr-layer-shell protocol.
/// if the user's compositor is in this list, we use the layer_shell/ folder.
const LAYER_SHELL_COMPOSITORS: &[&str] = &[
    "hyprland",
    "sway",
    "wayfire",
    "river",
    "labwc",
    "cage",
    "niri",
    "dwl",
];

/// detect the active compositor and its capabilities by reading
/// standard linux environment variables.
///
/// detection order:
/// 1. check XDG_SESSION_TYPE for wayland vs x11
/// 2. if wayland, check XDG_CURRENT_DESKTOP and compositor-specific env vars
///    to determine if layer-shell is supported
/// 3. if x11, return X11 capability (all x11 WMs support EWMH)
pub fn detect() -> ShellCapability {
    let session_type = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_lowercase();

    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let desktop_lower = desktop.to_lowercase();

    match session_type.as_str() {
        "wayland" => detect_wayland_compositor(&desktop, &desktop_lower),
        "x11" | "tty" => {
            println!("[detection] x11 session detected: {}", desktop);
            ShellCapability::X11 { desktop }
        }
        _ => {
            // fallback: probe for wayland or x11 display env vars
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                return detect_wayland_compositor(&desktop, &desktop_lower);
            }
            if std::env::var("DISPLAY").is_ok() {
                println!("[detection] x11 detected via DISPLAY env");
                return ShellCapability::X11 { desktop };
            }
            println!("[detection] could not detect display server");
            ShellCapability::Unknown
        }
    }
}

/// determine which wayland compositor is running and whether it
/// supports wlr-layer-shell. gnome/mutter is the only major
/// compositor that refuses to implement it.
fn detect_wayland_compositor(desktop: &str, desktop_lower: &str) -> ShellCapability {
    // check for known wlroots-based compositors (all support layer-shell)
    for &compositor in LAYER_SHELL_COMPOSITORS {
        if desktop_lower.contains(compositor) {
            println!("[detection] layer-shell compositor: {}", desktop);
            return ShellCapability::LayerShell {
                compositor_name: compositor.to_string(),
            };
        }
    }

    // hyprland sets its own env var even if XDG_CURRENT_DESKTOP is weird
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        println!("[detection] hyprland detected via instance signature");
        return ShellCapability::LayerShell {
            compositor_name: "hyprland".to_string(),
        };
    }

    // kde plasma (kwin) now supports layer-shell
    if desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
        println!("[detection] kde plasma detected (kwin, layer-shell supported)");
        return ShellCapability::LayerShell {
            compositor_name: "kwin".to_string(),
        };
    }

    // cosmic (pop!_os) supports layer-shell via smithay
    if desktop_lower.contains("cosmic") {
        println!("[detection] cosmic detected (layer-shell supported)");
        return ShellCapability::LayerShell {
            compositor_name: "cosmic".to_string(),
        };
    }

    // gnome / mutter - the one compositor that REFUSES layer-shell
    if desktop_lower.contains("gnome") || desktop_lower.contains("ubuntu") {
        println!("[detection] gnome/mutter detected (NO layer-shell, using fallback)");
        return ShellCapability::Mutter;
    }

    // unknown wayland compositor - optimistically try layer-shell
    // since most modern compositors support it
    println!(
        "[detection] unknown wayland compositor: '{}', attempting layer-shell",
        desktop
    );
    ShellCapability::LayerShell {
        compositor_name: format!("unknown ({})", desktop),
    }
}
