// ipc - unix domain socket communication between tauri app and sidecar
//
// the main tauri app (ui_app) spawns cl-video-player as a separate process.
// they communicate over unix domain sockets using a simple line-based
// text protocol. this is the linux equivalent of windows named pipes.
//
// protocol commands:
//   PLAY           - resume playback
//   PAUSE          - pause playback
//   STOP           - graceful shutdown
//   SWITCH <path>  - switch to a different video file
//   RELOAD         - reload current content (for scene renderer)
//   CAPTURE <path> - capture a preview frame to the given path

/// start an ipc listener on a unix domain socket.
/// runs in a background thread, calls the handler for each command received.
/// the sidecar binary (cl_vp.rs) calls this to listen for commands
/// from the main tauri app.
#[cfg(target_os = "linux")]
pub fn start_listener<F>(socket_path: &str, handler: F)
where
    F: Fn(&str) + Send + 'static,
{
    use std::io::BufRead;
    use std::os::unix::net::UnixListener;

    let socket_path = socket_path.to_string();

    // clean up any stale socket file from a previous crash
    let _ = std::fs::remove_file(&socket_path);

    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => {
            println!("[ipc] failed to bind socket '{}': {}", socket_path, e);
            return;
        }
    };

    println!("[ipc] listening on {}", socket_path);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let reader = std::io::BufReader::new(stream);
                    for line in reader.lines() {
                        match line {
                            Ok(cmd) => {
                                let cmd = cmd.trim().to_string();
                                if cmd.is_empty() {
                                    continue;
                                }
                                println!("[ipc] received: {}", cmd);

                                if cmd == "STOP" {
                                    handler(&cmd);
                                    println!("[ipc] stop received, shutting down listener");
                                    let _ = std::fs::remove_file(&socket_path);
                                    return;
                                }

                                handler(&cmd);
                            }
                            Err(e) => {
                                println!("[ipc] read error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("[ipc] connection error: {}", e);
                }
            }
        }
    });
}

/// send a command to a running sidecar over its unix domain socket.
/// the main tauri app calls this to send STOP, PAUSE, etc.
#[cfg(target_os = "linux")]
pub fn send_command(socket_path: &str, command: &str) {
    use std::io::Write;
    use std::os::unix::net::UnixStream;

    match UnixStream::connect(socket_path) {
        Ok(mut stream) => {
            let msg = format!("{}\n", command);
            let _ = stream.write_all(msg.as_bytes());
            let _ = stream.flush();
        }
        Err(e) => {
            println!("[ipc] send to '{}' failed: {}", socket_path, e);
        }
    }
}

// dev stubs for compiling on windows (the code editor runs here)
#[cfg(not(target_os = "linux"))]
pub fn start_listener<F>(socket_path: &str, _handler: F)
where
    F: Fn(&str) + Send + 'static,
{
    println!("[ipc] stub (not linux): would listen on {}", socket_path);
}

#[cfg(not(target_os = "linux"))]
pub fn send_command(socket_path: &str, command: &str) {
    println!(
        "[ipc] stub (not linux): {} -> {}",
        socket_path, command
    );
}
