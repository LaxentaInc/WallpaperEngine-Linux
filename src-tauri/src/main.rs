// colorwall linux - main tauri entry point
//
// this is the thinnest possible entry point. it wires up tauri plugins,
// registers UI commands, and starts the app. all logic lives in the
// library modules (core/, platform/, ui/).

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // commands re-exported from ui/mod.rs for clean access
            colorwall_linux_lib::ui::cmd_set_video_wallpaper,
            colorwall_linux_lib::ui::cmd_stop_wallpaper,
            colorwall_linux_lib::ui::cmd_get_display_info,
        ])
        .setup(|app| {
            println!(
                "[colorwall] starting colorwall linux v{}",
                env!("CARGO_PKG_VERSION")
            );

            let window = app.get_webview_window("main").unwrap();

            // hide on close instead of quitting (same pattern as windows version)
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                    println!("[colorwall] window hidden to tray");
                }
            });

            let _ = window.show();

            // detect and log the compositor at startup
            let shell = colorwall_linux_lib::platform::linux::shared::detection::detect();
            println!("[colorwall] detected shell: {:?}", shell);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running colorwall linux");
}
