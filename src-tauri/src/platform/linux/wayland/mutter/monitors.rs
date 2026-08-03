// mutter::monitors - gnome monitor enumeration
//
// gnome/mutter exposes monitor information through its own dbus interface:
// org.gnome.Mutter.DisplayConfig
//
// alternatively, since we're using xwayland fallback, we might be able
// to enumerate monitors via XRandR on the xwayland display, which would
// let us reuse the x11::monitors code.
//
// if the xwayland approach gives stale or incorrect monitor info
// (gnome sometimes virtualizes xwayland outputs), we fall back to
// the dbus approach.

use crate::platform::linux::shared::types::MonitorInfo;

/// enumerate monitors on gnome/mutter.
/// tries xrandr under xwayland first, falls back to dbus if needed.
pub fn enumerate() -> Result<Vec<MonitorInfo>, String> {
    // todo: implement
    // approach 1: XRandR on xwayland display (reuse x11::monitors)
    // approach 2: dbus org.gnome.Mutter.DisplayConfig.GetCurrentState
    println!("[mutter::monitors] enumerating gnome displays");
    Err("mutter monitor enumeration not implemented yet".to_string())
}
