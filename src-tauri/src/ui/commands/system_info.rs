// ui::commands::system_info - system information commands
//
// provides the frontend with information about the linux display
// environment: which compositor is running, which monitors are
// connected, and what capabilities are available.

use crate::platform::linux::shared::detection;

/// get display server info for the frontend.
/// returns the detected shell capability and relevant env vars.
#[tauri::command]
pub fn cmd_get_display_info() -> Result<serde_json::Value, String> {
    let shell = detection::detect();
    Ok(serde_json::json!({
        "shell_capability": format!("{:?}", shell),
        "session_type": std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
        "desktop": std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
    }))
}
