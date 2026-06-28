// layer_shell::surface - create a desktop-background wayland surface
//
// uses layershellev to request a wlr-layer-shell surface at LAYER_BACKGROUND.
// this surface sits behind all windows and desktop icons, exactly like
// the WorkerW hack on windows but using a proper compositor protocol.
//
// the surface is fullscreen on the target monitor, has no keyboard
// interactivity (clicks pass through), and exposes a window id that
// mpv.rs can render into.

use crate::platform::linux::shared::surface_trait::DesktopSurface;
use crate::platform::linux::shared::types::MonitorInfo;

pub struct LayerShellSurface {
    // todo: store the layershellev window/event loop state here
    // this will hold the wl_surface, layer_surface, and wl_display
    _monitor: MonitorInfo,
}

impl DesktopSurface for LayerShellSurface {
    fn create(monitor: &MonitorInfo) -> Result<Self, String> {
        // todo: implement using layershellev crate
        // 1. connect to the wayland display
        // 2. request a layer surface with Layer::Background
        // 3. set anchor to all edges (fullscreen)
        // 4. set exclusive zone to -1 (don't push other surfaces)
        // 5. set keyboard interactivity to None (clicks pass through)
        // 6. commit the surface
        println!(
            "[layer_shell] creating background surface on monitor '{}'",
            monitor.name
        );
        Err("layer_shell surface not implemented yet".to_string())
    }

    fn window_id(&self) -> u64 {
        // todo: return the wayland surface id or x11 window id
        // that mpv can use as its --wid render target
        0
    }

    fn run_event_loop(&mut self) -> Result<(), String> {
        // todo: pump wayland events (wl_display.dispatch)
        // this blocks the thread and keeps the surface alive
        Err("event loop not implemented yet".to_string())
    }

    fn destroy(&mut self) {
        // todo: destroy the layer surface and disconnect from wayland
        println!("[layer_shell] destroying surface");
    }
}
