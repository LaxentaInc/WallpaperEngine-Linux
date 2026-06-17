// cl-scene-renderer — sidecar binary for .colorwall scene composition
//
// this is the linux equivalent of cw-render.exe.
// it composites multiple layers (video, shader, particle, image)
// into a single output rendered via vulkan/wgpu.
//
// shares the same display server abstraction as cl-video-player
// for creating the background window.

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut scene_path = String::new();
    let mut monitor_id = "primary".to_string();
    let mut socket_path = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--scene" => {
                i += 1;
                if i < args.len() {
                    scene_path = args[i].clone();
                }
            }
            "--monitor" => {
                i += 1;
                if i < args.len() {
                    monitor_id = args[i].clone();
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
    println!("║      cl-scene-renderer — colorwall linux         ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("[renderer] scene:   {}", scene_path);
    println!("[renderer] monitor: {}", monitor_id);
    println!("[renderer] socket:  {}", socket_path);

    if scene_path.is_empty() {
        eprintln!("[renderer] error: no scene path provided (--scene <path>)");
        std::process::exit(1);
    }

    // todo: phase 3 implementation
    // 1. create background window (same display server abstraction as video player)
    // 2. initialize wgpu/vulkan renderer
    // 3. load scene.json and set up layer pipeline
    // 4. enter render loop
    // 5. listen for ipc commands (RELOAD, STOP, UPDATE, CAPTURE)

    println!("[renderer] scene renderer not yet implemented");
    println!("[renderer] this will be built in phase 3");

    if !socket_path.is_empty() {
        colorwall_linux_lib::platform::ipc::start_ipc_listener(&socket_path, |cmd| {
            println!("[renderer] ipc command: {}", cmd);
            if cmd == "STOP" {
                std::process::exit(0);
            }
        });
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
