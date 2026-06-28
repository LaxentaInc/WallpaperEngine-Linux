use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::window::Window;
use super::DesktopSurface;

pub struct X11Surface {
    window: Window,
}

impl X11Surface {
    pub fn new() -> Result<Self, String> {
        // TODO (RESEARCH): how to use winit to create an X11 window,
        // and how to set the _NET_WM_WINDOW_TYPE_DESKTOP X11 property on it.
        unimplemented!()
    }
}

impl DesktopSurface for X11Surface {
    fn window(&self) -> &Window {
        &self.window
    }

    fn set_background_layer(&self) {
        // TODO: implement X11-specific z-index lowering (_NET_WM_STATE_BELOW)
    }
}

// delegate the raw handles to the inner winit window so libmpv can attach to it
unsafe impl HasRawWindowHandle for X11Surface {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for X11Surface {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}
