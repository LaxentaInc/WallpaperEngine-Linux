// layer_shell::egl - EGL context management
//
// handles initializing an EGL display from a raw Wayland display pointer,
// creating a surfaceless EGL context, and binding it to a wl_egl_window.

use khronos_egl as egl;
use std::ffi::c_void;
use std::ptr;

pub struct EglContext {
    pub egl: egl::DynamicInstance<egl::EGL1_4>,
    pub display: egl::Display,
    pub config: egl::Config,
    pub context: egl::Context,
    pub surface: Option<egl::Surface>,
}

impl EglContext {
    /// Initialize EGL from a raw wayland display pointer
    pub fn new(wl_display_ptr: *mut c_void) -> Result<Self, String> {
        let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required() }
            .map_err(|e| format!("Failed to load EGL: {:?}", e))?;

        // 1. Get EGL display from Wayland display
        let display = unsafe { egl.get_display(wl_display_ptr) }
            .ok_or("Failed to get EGL display from Wayland display")?;

        // 2. Initialize EGL
        egl.initialize(display)
            .map_err(|e| format!("Failed to initialize EGL: {:?}", e))?;

        // 3. Choose EGL config (RGBA8888, OpenGL)
        let attrib_list = [
            egl::SURFACE_TYPE, egl::WINDOW_BIT,
            egl::RED_SIZE, 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE, 8,
            egl::ALPHA_SIZE, 8,
            egl::RENDERABLE_TYPE, egl::OPENGL_BIT,
            egl::NONE,
        ];
        
        // ensure OpenGL API is bound
        egl.bind_api(egl::OPENGL_API).map_err(|e| format!("Failed to bind OpenGL API: {:?}", e))?;

        let config = egl.choose_first_config(display, &attrib_list)
            .map_err(|e| format!("Failed to choose EGL config: {:?}", e))?
            .ok_or("No matching EGL config found")?;

        // 4. Create EGL context (surfaceless initially)
        let context_attribs = [
            egl::CONTEXT_CLIENT_VERSION, 2, // We want at least GL 2
            egl::NONE,
        ];
        
        let context = egl.create_context(display, config, None, &context_attribs)
            .map_err(|e| format!("Failed to create EGL context: {:?}", e))?;

        // Make context current with no surface yet (MPV needs this to init OpenGL)
        egl.make_current(display, None, None, Some(context))
            .map_err(|e| format!("Failed to make EGL context current (surfaceless): {:?}", e))?;

        Ok(Self {
            egl,
            display,
            config,
            context,
            surface: None,
        })
    }

    /// Creates an EGL surface from a wl_egl_window pointer and makes it current
    pub fn create_window_surface(&mut self, wl_egl_window_ptr: *mut c_void) -> Result<(), String> {
        let surface = unsafe { self.egl.create_window_surface(self.display, self.config, wl_egl_window_ptr, None) }
            .map_err(|e| format!("Failed to create EGL window surface: {:?}", e))?;
        
        self.egl.make_current(self.display, Some(surface), Some(surface), Some(self.context))
            .map_err(|e| format!("Failed to make EGL window surface current: {:?}", e))?;
            
        self.surface = Some(surface);
        Ok(())
    }

    /// Swaps the EGL buffers (present frame)
    pub fn swap_buffers(&self) -> Result<(), String> {
        if let Some(surface) = self.surface {
            self.egl.swap_buffers(self.display, surface)
                .map_err(|e| format!("Failed to swap EGL buffers: {:?}", e))?;
        }
        Ok(())
    }

    /// Wrapper for mpv get_proc_address
    pub fn get_proc_address(ctx: &Self, name: &str) -> *mut c_void {
        // egl exposes get_proc_address which returns Option<extern "C" fn()>
        // we need to cast it to *mut c_void for libmpv
        if let Some(func) = ctx.egl.get_proc_address(name) {
            func as usize as *mut c_void
        } else {
            ptr::null_mut()
        }
    }
}

impl Drop for EglContext {
    fn drop(&mut self) {
        let _ = self.egl.make_current(self.display, None, None, None);
        if let Some(surface) = self.surface {
            let _ = self.egl.destroy_surface(self.display, surface);
        }
        let _ = self.egl.destroy_context(self.display, self.context);
        let _ = self.egl.terminate(self.display);
    }
}
