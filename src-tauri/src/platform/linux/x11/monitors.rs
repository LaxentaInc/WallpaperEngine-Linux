// x11::monitors - X11 monitor enumeration via XRandR
//
// on X11, monitors are enumerated using the XRandR (X Resize and Rotate)
// extension. this gives us: output name, resolution, position in the
// virtual screen coordinate space, rotation, and connection status.
//
// XRandR is universally supported across all X11 window managers.

use crate::platform::linux::shared::types::MonitorInfo;

/// enumerate all connected monitors via the XRandR extension.
/// returns a vec of MonitorInfo with positions in the virtual screen space.
pub fn enumerate() -> Result<Vec<MonitorInfo>, String> {
    // todo: implement using x11rb or xrandr bindings
    // 1. open x11 connection
    // 2. query XRandR screen resources
    // 3. for each output, get: name, crtc, mode (resolution), position
    // 4. determine primary output
    // 5. map into MonitorInfo structs
    println!("[x11::monitors] enumerating XRandR outputs");
    Err("x11 monitor enumeration not implemented yet".to_string())
}
