// platform::linux::layer_shell - wlr-layer-shell compositor support
//
// this folder handles ALL compositors that implement the
// wlr-layer-shell-unstable-v1 protocol. as of 2025, this covers:
//   - hyprland, sway, river, wayfire, labwc, niri, dwl (wlroots-based)
//   - kde plasma (kwin)
//   - cosmic (pop!_os, smithay-based)
//   - gamescope (steamos)
//
// uses the layershellev crate to create a surface pinned to
// LAYER_BACKGROUND with no keyboard interactivity, so clicks
// pass through to the desktop icons above.

pub mod surface;
pub mod monitors;
