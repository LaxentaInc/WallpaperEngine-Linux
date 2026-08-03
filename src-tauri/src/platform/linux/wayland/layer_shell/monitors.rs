// wayland::monitors - wayland monitor enumeration
//
// on wayland, monitors are represented as wl_output globals.
// each wl_output provides: name, make, model, resolution,
// refresh rate, position in the virtual coordinate space, and scale.
//
// the xdg-output-unstable-v1 extension adds logical size/position
// which accounts for fractional scaling.
//
// this module enumerates all wl_outputs and maps them into
// the shared MonitorInfo struct.

use crate::platform::linux::shared::types::MonitorInfo;

/// enumerate all connected monitors via wayland's wl_output protocol.
/// returns a vec of MonitorInfo with positions in the virtual coordinate space.
pub fn enumerate() -> Result<Vec<MonitorInfo>, String> {
    // todo: implement using wayland-client crate
    // 1. connect to wl_display
    // 2. do a registry roundtrip to discover wl_output globals
    // 3. for each wl_output, collect name, geometry, mode, scale
    // 4. optionally bind xdg-output-manager for logical coordinates
    // 5. map everything into MonitorInfo structs
    println!("[wayland::monitors] enumerating wayland outputs");
    Err("wayland monitor enumeration not implemented yet".to_string())
}
