// engine, spawns and manages the cl-video-player sidecar binary
//
// mirrors the windows engine.rs pattern: the tauri app doesn't render
// the wallpaper itself. it spawns a separate process (cl-video-player)
// that handles display server integration, vulkan surface creation,
// and hardware-accelerated video decode.
//
// communication happens over unix domain sockets (equivalent to
// windows named pipes).

use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Mutex;

use super::display;

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
    stop_wallpaper_on_monitor(target);

    // detect display server to pass to the sidecar
    let display_server = display::detect();
    println!(
        "[engine] setting video wallpaper on '{}' (display: {})",
        target,
        display_server.name()
    );

    // resolve the sidecar binary path
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("failed to get current exe: {}", e))?
        .parent()
        .ok_or("failed to get exe parent dir")?
        .to_path_buf();

    // on linux the binary has no extension, on windows it would be .exe
    let player_name = if cfg!(target_os = "windows") {
        "cl-video-player.exe"
    } else {
        "cl-video-player"
    };
    let player_path = exe_dir.join(player_name);

    // during development, the binary might not exist yet
    if !player_path.exists() {
        println!(
            "[engine] warning: cl-video-player not found at {}, skipping",
            player_path.display()
        );
        return Err(format!(
            "cl-video-player not found at {} (build it with: cargo build --bin cl-video-player)",
            player_path.display()
        ));
    }

    // socket path for ipc (unix domain socket on linux, tcp on windows for dev)
    let socket_path = get_socket_path(target);

    // clean up stale socket file
    let _ = std::fs::remove_file(&socket_path);

    // resolve video path
    let video_path_abs = std::fs::canonicalize(video_path)
        .map_err(|e| format!("failed to resolve video path: {}", e))?;

    println!(
        "[engine] spawning cl-video-player: {} --video {} --monitor {} --display-server {}",
        player_path.display(),
        video_path_abs.display(),
        target,
        display_server.name()
    );

    let child = Command::new(&player_path)
        .arg("--video")
        .arg(video_path_abs.to_string_lossy().as_ref())
        .arg("--monitor")
        .arg(target)
        .arg("--display-server")
        .arg(display_server.name())
        .arg("--socket")
        .arg(&socket_path)
        .spawn()
        .map_err(|e| format!("failed to spawn cl-video-player: {}", e))?;

    let pid = child.id();
    println!("[engine] cl-video-player spawned with pid: {}", pid);

    // track the process
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
pub fn stop_wallpaper() -> Result<(), String> {
    let mut processes = PLAYER_PROCESSES.lock().unwrap();
    let monitors: Vec<String> = processes.keys().cloned().collect();

    for monitor in monitors {
        if let Some(mut process) = processes.remove(&monitor) {
            println!("[engine] stopping player on '{}'", monitor);
            send_ipc_command(&process.socket_path, "STOP");
            std::thread::sleep(std::time::Duration::from_millis(200));
            let _ = process.child.kill();
            let _ = process.child.wait();
            let _ = std::fs::remove_file(&process.socket_path);
        }
    }

    println!("[engine] all players stopped");
    Ok(())
}

/// stop the wallpaper on a specific monitor
pub fn stop_wallpaper_on_monitor(monitor_id: &str) {
    let mut processes = PLAYER_PROCESSES.lock().unwrap();
    if let Some(mut process) = processes.remove(monitor_id) {
        println!("[engine] stopping player on '{}'", monitor_id);
        send_ipc_command(&process.socket_path, "STOP");
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = process.child.kill();
        let _ = process.child.wait();
        let _ = std::fs::remove_file(&process.socket_path);
    }
}

/// get the socket path for a monitor's player ipc
fn get_socket_path(monitor_id: &str) -> String {
    if cfg!(target_os = "linux") {
        // use XDG_RUNTIME_DIR on linux (typically /run/user/1000/)
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/tmp".to_string());
        format!("{}/colorwall-player-{}.sock", runtime_dir, monitor_id)
    } else {
        // fallback for windows development: use temp dir
        let tmp = std::env::temp_dir();
        format!(
            "{}\\colorwall-player-{}.sock",
            tmp.display(),
            monitor_id
        )
    }
}

/// send a command to a player process over ipc
fn send_ipc_command(socket_path: &str, command: &str) {
    // on linux: unix domain socket
    // on windows (dev only): just log it
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        match UnixStream::connect(socket_path) {
            Ok(mut stream) => {
                let msg = format!("{}\n", command);
                let _ = stream.write_all(msg.as_bytes());
                let _ = stream.flush();
            }
            Err(e) => {
                println!("[engine] ipc send to '{}': {}", socket_path, e);
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // dev stub: just log the command
        println!("[engine] ipc stub (not linux): {} -> {}", socket_path, command);
    }
}
