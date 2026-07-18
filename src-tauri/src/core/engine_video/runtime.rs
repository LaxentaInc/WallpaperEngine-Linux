// core::engine_video::launcher - spawn cl-video-player
//
// handles resolving video paths, detecting the shell, and executing
// the child process with the correct arguments.

use std::process::Command;

use super::paths;
use super::shutdown;
use super::process_state::{PlayerProcess, PLAYER_PROCESSES};
use crate::platform::linux::shared::detection;
use crate::platform::linux::shared::types::ShellCapability;

/// start a video wallpaper on a specific monitor
pub fn set_video_wallpaper(
    _app: &tauri::AppHandle,
    video_path: &str,
    monitor_id: Option<&str>,
) -> Result<(), String> {
    let target = monitor_id.unwrap_or("primary");

    // stop any existing player on this monitor
    shutdown::stop_on_monitor(target);

    // detect which shell capability to 
    // the shared folders detection.rs is called here
    // and erm this is the orchestrator that wires everything up.
    let shell = detection::detect();
    let shell_name = match &shell {
        ShellCapability::LayerShell { compositor_name } => {
            format!("layer-shell ({})", compositor_name)
        }
        ShellCapability::X11 { desktop } => format!("x11 ({})", desktop),
        ShellCapability::Mutter => "mutter".to_string(),
        ShellCapability::Unknown => "unknown".to_string(),
    };
    println!(
        "[engine_video] setting wallpaper on '{}' (shell: {})",
        target, shell_name
    );

    let player_path = paths::get_player_path()?;

    // unix domain socket path for ipc
    let socket_path = paths::get_socket_path(target);
    let _ = std::fs::remove_file(&socket_path);

    // resolve video path to absolute
    let video_path_abs = std::fs::canonicalize(video_path)
        .map_err(|e| format!("failed to resolve video path: {}", e))?;

    // serialize the shell capability as a CLI arg
    let shell_arg = match &shell {
        ShellCapability::LayerShell { .. } => "layer-shell",
        ShellCapability::X11 { .. } => "x11",
        ShellCapability::Mutter => "mutter",
        ShellCapability::Unknown => "unknown",
    };

    println!(
        "[engine_video] spawning: {} --video {} --monitor {} --shell {} --socket {}",
        player_path.display(),
        video_path_abs.display(),
        target,
        shell_arg,
        socket_path
    );

    let child = Command::new(&player_path)
        .arg("--video")
        .arg(video_path_abs.to_string_lossy().as_ref())
        .arg("--monitor")
        .arg(target)
        .arg("--shell")
        .arg(shell_arg)
        .arg("--socket")
        .arg(&socket_path)
        .spawn()
        .map_err(|e| format!("failed to spawn cl-video-player: {}", e))?;

    let pid = child.id();
    println!("[engine_video] cl-video-player spawned with pid: {}", pid);

    let mut processes = PLAYER_PROCESSES.lock().unwrap();
    processes.insert(
        target.to_string(),
        PlayerProcess {
            child,
            socket_path: socket_path.clone(),
            video_path: video_path.to_string(),
        },
    );

    Ok(())
}
