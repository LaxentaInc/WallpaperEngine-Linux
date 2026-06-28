// core::engine_video::shutdown - cleanup logic for sidecars
//
// handles gracefully stopping running cl-video-player instances
// via IPC, then forcefully killing them if they hang.

use super::process_state::PLAYER_PROCESSES;
use crate::platform::linux::shared::ipc;

/// stop all wallpaper player processes across all monitors
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
