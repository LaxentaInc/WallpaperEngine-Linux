// core::engine_video - video player sidecar lifecycle management
//
// this module is responsible for spawning, tracking, and killing
// the cl-video-player sidecar binary. the tauri app never renders
// video itself; it delegates to this separate process.
//
// mirrors: core/engine_video/ in the windows version

pub mod process;
