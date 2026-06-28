// core::engine_video::paths - path resolution for the video engine
//
// handles resolving the sidecar binary path and generating
// unique unix domain socket paths for ipc.

/// resolve the sidecar binary path (sits next to the main binary)
pub fn get_player_path() -> Result<std::path::PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("failed to get current exe: {}", e))?
        .parent()
        .ok_or("failed to get exe parent dir")?
        .to_path_buf();

    let player_path = exe_dir.join("cl-video-player");

    if !player_path.exists() {
        return Err(format!(
            "cl-video-player not found at {} (build with: cargo build --bin cl-video-player)",
            player_path.display()
        ));
    }

    Ok(player_path)
}

/// get the unix domain socket path for a monitor's player ipc.
/// uses XDG_RUNTIME_DIR (typically /run/user/1000/) which is the
/// standard location for per-user runtime files on linux.
pub fn get_socket_path(monitor_id: &str) -> String {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/colorwall-player-{}.sock", runtime_dir, monitor_id)
}
