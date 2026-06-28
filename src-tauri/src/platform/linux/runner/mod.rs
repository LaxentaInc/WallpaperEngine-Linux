// platform::linux::video - video playback logic (shell-agnostic)
//
// this module handles pure video playback using libmpv.
// it receives a raw window handle from whichever shell created the
// desktop surface and renders decoded video frames into it.
//
// mpv.rs does NOT know or care whether it's running on wayland,
// x11, or gnome. it just takes a window id and plays.

pub mod config;
pub mod mpv;