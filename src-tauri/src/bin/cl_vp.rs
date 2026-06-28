// cl-video-player - sidecar binary entry point
//
// this is the standalone process that the tauri app spawns to render
// video wallpapers. it is intentionally isolated: if it crashes,
// the main UI keeps running and can restart it.
//
// this file BUILDS IT ALL TOGETHER:
// 1. parses CLI args from the tauri app
// 2. uses the --shell arg to pick the correct compositor implementation
// 3. creates a desktop-background surface using that implementation
// 4. attaches mpv to the surface for hardware-accelerated video decode
// 5. starts the IPC listener to receive commands from the tauri app
//
// the actual implementation code lives in platform/linux/.
// this file is just the orchestrator that wires everything up.

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut video_path = String::new();
    let mut monitor_id = "primary".to_string();
    let mut shell_type = String::new();
    let mut socket_path = String::new();

    // parse cli args passed by core::engine_video::process
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--video" => {
                i += 1;
                if i < args.len() {
                    video_path = args[i].clone();
                }
            }
            "--monitor" => {
                i += 1;
                if i < args.len() {
                    monitor_id = args[i].clone();
                }
            }
            "--shell" => {
                i += 1;
                if i < args.len() {
                    shell_type = args[i].clone();
                }
            }
            "--socket" => {
                i += 1;
                if i < args.len() {
                    socket_path = args[i].clone();
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("╔══════════════════════════════════════════════════╗");
    println!("║        cl-video-player — colorwall linux         ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("[player] video:   {}", video_path);
    println!("[player] monitor: {}", monitor_id);
    println!("[player] shell:   {}", shell_type);
    println!("[player] socket:  {}", socket_path);

    if video_path.is_empty() {
        eprintln!("[player] error: no video path provided (--video <path>)");
        std::process::exit(1);
    }

    // step 1: create the desktop surface using the correct compositor implementation
    // the --shell arg was set by core::engine_video::process based on detection results
    match shell_type.as_str() {
        "layer-shell" => {
            println!("[player] using platform/linux/layer_shell/ implementation");
            // todo: call platform::linux::layer_shell::surface::LayerShellSurface::create()
        }
        "x11" => {
            println!("[player] using platform/linux/x11/ implementation");
            // todo: call platform::linux::x11::surface::X11Surface::create()
        }
        "mutter" => {
            println!("[player] using platform/linux/mutter/ implementation");
            // todo: call platform::linux::mutter::surface::MutterSurface::create()
        }
        _ => {
            eprintln!("[player] error: unknown shell type '{}'. expected: layer-shell, x11, mutter", shell_type);
            std::process::exit(1);
        }
    }

    // step 2: start ipc listener for commands from the tauri app
    if !socket_path.is_empty() {
        colorwall_linux_lib::platform::linux::shared::ipc::start_listener(&socket_path, |cmd| {
            println!("[player] ipc command: {}", cmd);
            if cmd == "STOP" {
                println!("[player] shutting down");
                std::process::exit(0);
            }
        });
    }

    // step 3: initialize mpv on the surface (this blocks!)
    let config = colorwall_linux_lib::platform::linux::runner::config::MpvConfig {
        video_path,
        window_id: 0, // 0 = test mode, spawn floating window
        loop_playback: true,
        volume: 0,
    };

    if let Err(e) = colorwall_linux_lib::platform::linux::runner::mpv::initialize(&config) {
        eprintln!("[player] mpv error: {}", e);
        std::process::exit(1);
    }
}
