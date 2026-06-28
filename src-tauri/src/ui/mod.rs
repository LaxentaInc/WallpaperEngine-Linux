// ui - tauri command handlers for the react frontend
//
// every function the frontend can call lives under ui::commands.
// this mirrors the windows version's ui/commands/ folder.

pub mod commands;

// re-export all tauri commands at the ui:: level for cleaner imports in main.rs
pub use commands::video_ops::cmd_set_video_wallpaper;
pub use commands::video_ops::cmd_stop_wallpaper;
pub use commands::system_info::cmd_get_display_info;
