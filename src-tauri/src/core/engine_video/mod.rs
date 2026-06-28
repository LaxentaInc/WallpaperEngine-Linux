// core::engine_video - video player sidecar lifecycle management
//
// this module is responsible for spawning, tracking, and killing
// the cl-video-player sidecar binary. the tauri app never renders
// video itself; it delegates to this separate process.
//
// mirrors: core/engine_video/ in the windows version

pub mod process_state;
pub mod paths;
pub mod shutdown;
pub mod runtime;

// re-export the public API for the rest of the app to use
pub use runtime::set_video_wallpaper;
pub use shutdown::{stop_all, stop_on_monitor};
