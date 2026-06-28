// core::engine_video::process - spawn and manage cl-video-player
//
// this is the bridge between the tauri UI and the video player sidecar.
// when the user clicks "set wallpaper" in the react frontend, this module:
// 1. detects the display server (via platform::linux::shared::detection)
// 2. resolves the cl-video-player binary path
// 3. spawns it with CLI args (video path, monitor, display server, socket)
// 4. tracks the process so it can be killed later
//
// mirrors: core/engine_video/process.rs in the windows version

use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Mutex;

use crate::platform::linux::shared::detection;
use crate::platform::linux::shared::ipc;
use crate::platform::linux::shared::types::ShellCapability;

// global state for running player processes (one per monitor)
lazy_static::lazy_static! {
    static ref PLAYER_PROCESSES: Mutex<HashMap<String, PlayerProcess>> = Mutex::new(HashMap::new());
}

/// a running cl-video-player instance
struct PlayerProcess {
    child: Child,
    socket_path: String,
    #[allow(dead_code)]
    video_path: String,
}

/// start a video wallpaper on a specific monitor
pub fn set_video_wallpaper(
    _app: &tauri::AppHandle,
    video_path: &str,
    monitor_id: Option<&str>,
) -> Result<(), String> {
    let target = monitor_id.unwrap_or("primary");

    // stop any existing player on this monitor
    stop_on_monitor(target);

    // detect which shell capability to use
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

    // resolve the sidecar binary path (sits next to the main binary)
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("failed to get current exe: {}", e))?
        .parent()
        .ok_or("failed to get exe parent dir")?
        .to_path_buf();

    let player_path = exe_dir.join("cl-video-player");

    if !player_path.exists() {
        println!(
            "[engine_video] warning: cl-video-player not found at {}",
            player_path.display()
        );
        return Err(format!(
            "cl-video-player not found at {} (build with: cargo build --bin cl-video-player)",
            player_path.display()
        ));
    }

    // unix domain socket path for ipc
    let socket_path = get_socket_path(target);
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

/// stop all wallpaper player processes
pub fn stop_all() -> Result<(), String> {
    let mut processes = PLAYER_PROCESSES.lock().unwrap();
    let monitors: Vec<String> = processes.keys().cloned().collect();

    for monitor in monitors {
        if let Some(mut process) = processes.remove(&monitor) {
            println!("[engine_video] stopping player on '{}'", monitor);
            ipc::send_command(&process.socket_path, "STOP");
            std::thread::sleep(std::time::Duration::from_millis(200));
            let _ = process.child.kill();
            let _ = process.child.wait();
            let _ = std::fs::remove_file(&process.socket_path);
        }
    }

    println!("[engine_video] all players stopped");
    Ok(())
}

/// stop the wallpaper on a specific monitor
pub fn stop_on_monitor(monitor_id: &str) {
    let mut processes = PLAYER_PROCESSES.lock().unwrap();
    if let Some(mut process) = processes.remove(monitor_id) {
        println!("[engine_video] stopping player on '{}'", monitor_id);
        ipc::send_command(&process.socket_path, "STOP");
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = process.child.kill();
        let _ = process.child.wait();
        let _ = std::fs::remove_file(&process.socket_path);
    }
}

/// get the unix domain socket path for a monitor's player ipc.
/// uses XDG_RUNTIME_DIR (typically /run/user/1000/) which is the
/// standard location for per-user runtime files on linux.
fn get_socket_path(monitor_id: &str) -> String {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/colorwall-player-{}.sock", runtime_dir, monitor_id)
}
