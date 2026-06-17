// ipc — unix domain socket protocol for sidecar communication
//
// replaces windows named pipes with unix domain sockets.
// same line-based text protocol: command\n
//
// commands:
//   PLAY          - resume playback
//   PAUSE         - pause playback
//   STOP          - graceful shutdown
//   SWITCH <path> - switch to a different video
//   RELOAD        - reload current content (for scene renderer)
//   CAPTURE <path> - capture a preview frame to the given path

/// start an ipc listener on a unix domain socket.
/// runs in its own thread, calls the handler for each command received.
///
/// on windows (dev only) this is a no-op stub.
#[cfg(target_os = "linux")]
pub fn start_ipc_listener<F>(socket_path: &str, handler: F)
where
    F: Fn(&str) + Send + 'static,
{
    use std::io::BufRead;
    use std::os::unix::net::UnixListener;

    let socket_path = socket_path.to_string();

    // clean up stale socket
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

/// dev stub: log that we'd start a listener, but don't actually
/// bind a unix socket since we're on windows
#[cfg(not(target_os = "linux"))]
pub fn start_ipc_listener<F>(socket_path: &str, _handler: F)
where
    F: Fn(&str) + Send + 'static,
{
    println!("[ipc] stub (not linux): would listen on {}", socket_path);
}
