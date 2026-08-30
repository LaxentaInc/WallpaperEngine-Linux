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

        // load OpenGL function pointers from the EGL loader.
        // the `gl` crate is a static dispatch table that starts as all null pointers;
        // gl::load_with fills it by calling eglGetProcAddress for each GL symbol.
        // this must happen after make_current so the GL context is active.


        // So the explaination goes: OpenGL is an API specification, not a hardcoded library built into your executable.
        // The actual implementation of functions like glViewport lives inside your graphics card driver 
        // (Nvidia, AMD, Intel, or Mesa).
        // Because the operating system doesn't link these functions automatically at compile time,
        //  your program initializes them as empty, NULL pointers.If you call gl::Viewport while it is NULL, 
        // your program attempts to execute code at memory address 0, causing an immediate crash.
        // What gl::load_with fills them withThe gl::load_with function acts as a dynamic finder. It talks 
        // to the system's graphics driver to fetch the actual memory addresses of the compiled OpenGL functions
        //  and swaps out the NULL pointers with those real addresses.To do this, gl::load_with requires two things:
        // An active EGL/GL Context: The graphics driver will only reveal function addresses if a valid OpenGL 
        // context is currently active on the calling thread.
        // A loader function: You pass a helper function (like eglGetProcAddress) into gl::load_with.
        // loads       unsafe {
        //     gl::Viewport(0, 0, width, height);
        // }
        // with vieports coordinates by querying the driver as a medium and filling the x00 values that the code might try to access and get probable access violations? probably crash. ( in mpv )
        // path to: C:\Users\MY-PC\Documents\WallpaperEngine-Linux\src-tauri\src\platform\linux\runner
        gl::load_with(|name| {
            egl.get_proc_address(name)
                .map(|f| f as *const std::ffi::c_void)
                .unwrap_or(std::ptr::null())
        });

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
