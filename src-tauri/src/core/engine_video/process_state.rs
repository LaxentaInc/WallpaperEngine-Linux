// core::engine_video::state - global tracker for running sidecars
// for when the app is running, to load it in memory

// maintains a map of which monitor is currently running which
// cl-video-player process, along with their ipc sockets and pids.

use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;

// global state for running player processes (one per monitor)
lazy_static::lazy_static! {
    pub static ref PLAYER_PROCESSES: Mutex<HashMap<String, PlayerProcess>> = Mutex::new(HashMap::new());
}

/// a running cl-video-player instance
pub struct PlayerProcess {
    pub child: Child,
    pub socket_path: String,
    #[allow(dead_code)]
    pub video_path: String,
}
