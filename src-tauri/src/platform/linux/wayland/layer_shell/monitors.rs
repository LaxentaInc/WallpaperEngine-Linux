// wayland::monitors - wayland monitor enumeration
//
// queries the wayland compositor via layershellev's zxdg_output_manager implementation.
// maps logical dimensions, positions, and output names into the shared MonitorInfo struct.

use crate::platform::linux::shared::types::MonitorInfo;

/// enumerate all connected monitors via wayland's wl_output and xdg-output protocol.
/// returns a vec of MonitorInfo with positions in the virtual coordinate space.
pub fn enumerate() -> Result<Vec<MonitorInfo>, String> {
    tracing::info!("[wayland::monitors] querying compositor outputs via layershellev");

    // invoke our standalone monitor enumerator to query wl_output and zxdg_output_manager inside modifie crate of layershellev.
    // this avoids duplicate wayland connections or protocol crate mismatches.
    let xdg_outputs = layershellev::enumerate_monitors()
        .map_err(|e| format!("failed to enumerate wayland outputs: {:?}", e))?;

    let mut monitors = Vec::new();
    for (i, info) in xdg_outputs.into_iter().enumerate() {
        let (width, height) = info.get_logical_size();
        let (x, y) = info.get_position();
        let name = info.get_name().to_string();
        let display_name = if name.is_empty() {
            format!("display-{}", i + 1)
        } else {
            name.clone()
        };

        tracing::info!(
            "[wayland::monitors] found output {}: '{}' ({}x{}) at ({}, {})",
            i, display_name, width, height, x, y
        );

        // map each xdg-output into the unified MonitorInfo struct used by the tauri frontend.
        monitors.push(MonitorInfo {
            id: if name.is_empty() { format!("output-{}", i) } else { name.clone() },
            name: display_name,
            x,
            y,
            width: width.max(0) as u32,
            height: height.max(0) as u32,
            scale: 1.0,
            primary: i == 0,
        });
    }

    Ok(monitors)
}

