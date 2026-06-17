// cl-video-player — sidecar binary for video wallpaper playback
//
// this is the linux equivalent of wallpaper-player.exe.
// it runs as a separate process spawned by the tauri app.
//
// responsibilities:
// 1. detect display server (wayland layer-shell / x11 ewmh / gnome xwayland)
// 2. create a background-layer window on the target monitor
// 3. initialize libmpv with hardware-accelerated decode (va-api / nvdec)
// 4. render decoded frames to the window surface
// 5. listen for ipc commands over a unix domain socket
//
// the binary is intentionally standalone — if it crashes, the main
// tauri app detects it and can restart it without affecting the ui.

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut video_path = String::new();
    let mut monitor_id = "primary".to_string();
    let mut display_server = String::new();
    let mut socket_path = String::new();

    // parse cli args
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
            "--display-server" => {
                i += 1;
                if i < args.len() {
                    display_server = args[i].clone();
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
    println!("[player] video:          {}", video_path);
    println!("[player] monitor:        {}", monitor_id);
    println!("[player] display server: {}", display_server);
    println!("[player] socket:         {}", socket_path);

    if video_path.is_empty() {
        eprintln!("[player] error: no video path provided (--video <path>)");
        std::process::exit(1);
    }

    // todo: phase 1 implementation
    // 1. based on display_server, create the background window:
    //    - "wayland-wlroots" / "wayland-kde" → zwlr_layer_shell_v1
    //    - "wayland-gnome" / "x11"           → xwayland + ewmh hints
    //
    // 2. initialize libmpv with hardware decode:
    //    mpv_set_option_string(ctx, "hwdec", "auto");
    //    mpv_set_option_string(ctx, "vo", "gpu-next");
    //
    // 3. start the render loop
    //
    // 4. listen for ipc commands on the unix domain socket

    // placeholder: just hang until killed so the engine can track us
    println!("[player] waiting for implementation... (ctrl+c to exit)");

    // start ipc listener (this part works already)
    if !socket_path.is_empty() {
        colorwall_linux_lib::platform::ipc::start_ipc_listener(&socket_path, |cmd| {
            println!("[player] ipc command: {}", cmd);
            if cmd == "STOP" {
                println!("[player] shutting down");
                std::process::exit(0);
            }
        });
    }

    // block main thread
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
