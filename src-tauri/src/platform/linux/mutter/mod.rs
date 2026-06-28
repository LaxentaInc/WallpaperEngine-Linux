// platform::linux::mutter - gnome/mutter fallback support
//
// gnome's compositor (mutter) is the ONLY major wayland compositor
// that refuses to implement the wlr-layer-shell protocol.
// gnome's philosophy is that apps should not control window placement.
//
// possible approaches (all painful):
// 1. xwayland fallback: create an x11 window under xwayland and set
//    EWMH desktop hints. works but uses the x11 compatibility layer.
// 2. gnome shell extension: write a gnome-shell js extension that
//    creates a background actor. most reliable but requires the user
//    to install an extension.
// 3. dbus: use org.gnome.Shell.Extensions dbus interface to inject
//    a background surface. experimental.
//
// for now, we will start with approach 1 (xwayland fallback) since
// it requires zero user setup and reuses most of the x11/ code.

pub mod surface;
pub mod monitors;
