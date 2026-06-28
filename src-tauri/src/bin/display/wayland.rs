use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::window::Window;
use super::DesktopSurface;

pub struct WaylandSurface {
    window: Window,
}

impl WaylandSurface {
    pub fn new() -> Result<Self, String> {
        // TODO (RESEARCH): how to use layershellev to create a background layer shell.
        // Hint: WindowState::new().with_layer(Layer::Background).with_anchor(...)
        // You will bypass winit entirely here.
        unimplemented!()
    }
}

impl DesktopSurface for WaylandSurface {
    fn window(&self) -> &Window {
        &self.window
    }

    fn set_background_layer(&self) {
        // TODO: implement wayland-specific layer locking if not done during creation
    }
}

// delegate the raw handles to the inner winit window so libmpv can attach to it
unsafe impl HasRawWindowHandle for WaylandSurface {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for WaylandSurface {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}
