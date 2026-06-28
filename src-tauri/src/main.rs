// colorwall linux — main entry point
// the tauri application lifecycle and window management

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // -- platform commands --
            colorwall_linux::platform::cmd_set_video_wallpaper,
            colorwall_linux::platform::cmd_stop_wallpaper,
            colorwall_linux::platform::cmd_get_display_info,
        ])
        .setup(|app| {
            println!("[colorwall] starting colorwall linux v{}", env!("CARGO_PKG_VERSION"));

            let window = app.get_webview_window("main").unwrap();

            // hide on close (same pattern as windows version)
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                    println!("[colorwall] window hidden to tray");
                }
            });

            // show the window once the frontend is ready
            let _ = window.show();

            // detect display server and log it
            let display_info = colorwall_linux::platform::detect_display_server();
            println!("[colorwall] display server: {:?}", display_info);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running colorwall linux");
}
