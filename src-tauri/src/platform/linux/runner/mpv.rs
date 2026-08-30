// video::mpv - libmpv video playback engine
//
// pure mpv initialization and control. this module receives an EGL context
// and sets up the libmpv render API to paint frames via OpenGL onto the Wayland surface.

use super::config::MpvConfig;
use crate::platform::linux::wayland::layer_shell::egl::EglContext;
use layershellev::calloop::channel::Sender;
use crate::platform::linux::wayland::layer_shell::surface::PlayerMessage;
use libmpv2::{
    Mpv,
    render::{OpenGLInitParams, RenderParam, RenderParamApiType, mpv_render_update},
};
use std::ffi::c_void;
use std::time::Instant;

/// minimum interval between renders (~60fps).
/// prevents flooding the compositor when mpv fires update callbacks faster
/// than the display can present frames (especially brutal on software decoding in vms).
const MIN_FRAME_INTERVAL_MS: u128 = 16;

/// helper to unwrap the context pointer and pass to our EGL proc loader
fn mpv_get_proc_address(ctx: &*const c_void, name: &str) -> *mut c_void {
    let egl = unsafe { &*(*ctx as *const EglContext) };
    EglContext::get_proc_address(egl, name)
}

pub struct MpvPlayer {
    pub mpv: &'static Mpv,
    pub render_context: libmpv2::render::RenderContext<'static>,
    /// tracks the last time we actually rendered a frame to throttle
    /// the render loop to ~60fps and avoid flooding the compositor.
    last_render_time: Instant,
}

impl MpvPlayer {
    #[tracing::instrument(skip_all)]
    pub fn new(
        config: &MpvConfig,
        egl_context: &EglContext,
        event_sender: Sender<PlayerMessage>,
    ) -> Result<Self, String> {
        tracing::info!("[mpv] initializing libmpv context with EGL render API...");
        
        let mpv = Mpv::with_initializer(|init| {
            // lock down mpv to prevent it from spawning its own window
            let _ = init.set_property("config", "no");
            let _ = init.set_property("force-window", "no");
            
            let _ = init.set_property("vo", "libmpv");
            let _ = init.set_property("hwdec", "auto-safe");
            
            let _ = init.set_property("profile", "fast");
            let _ = init.set_property("vd-lavc-fast", "yes");
            let _ = init.set_property("vd-lavc-skiploopfilter", "all");
            let _ = init.set_property("osc", "no");
            let _ = init.set_property("window-dragging", "no");
            let _ = init.set_property("input-default-bindings", "no");
            let _ = init.set_property("audio", "no");
            let _ = init.set_property("border", "no");
            // we will Loop by default
            // TODO: KEEP THIS BUT REMOVE THIS FROM SETTINGS. SO No more config reads to set the property flag. 
            if config.loop_playback {
                let _ = init.set_property("loop-file", "inf");
            }
            init.set_property("volume", config.volume as i64).unwrap();
            Ok(())
        }).map_err(|e| format!("failed to create mpv context: {}", e))?;

        let mpv_ref: &'static Mpv = Box::leak(Box::new(mpv));
        let ctx_ptr = egl_context as *const _ as *const c_void;
        
        let mut render_context = mpv_ref
            .create_render_context(vec![
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(OpenGLInitParams {
                    get_proc_address: mpv_get_proc_address,
                    ctx: ctx_ptr,
                }),
            ])
            .map_err(|e| format!("Failed to create mpv render context: {:?}", e))?;

        // Register the update callback which sends a message through the calloop channel
        render_context.set_update_callback(move || {
            let _ = event_sender.send(PlayerMessage::MpvRedrawRequested);
        });

        tracing::info!("loading video: {}", config.video_path);
        mpv_ref.command("loadfile", &[&config.video_path])
            .map_err(|e| format!("failed to load video: {}", e))?;

        Ok(Self {
            mpv: mpv_ref,
            render_context,
            last_render_time: Instant::now(),
        })
    }

    /// renders the current mpv frame to the EGL surface.
    ///
    /// this method implements three critical safeguards from mpvpaper's architecture:
    /// 1. calls render_context.update() FIRST to check if a new frame is actually
    ///    pending. if not, skips the render entirely to avoid redundant GPU work.
    /// 2. calls glViewport before render to ensure the OpenGL framebuffer matches
    ///    the actual EGL surface dimensions (fixes video not filling monitor).
    /// 3. calls report_swap() AFTER eglSwapBuffers to tell libmpv the buffer was
    ///    presented, releasing GL fence objects and preventing memory leaks.
    #[tracing::instrument(skip_all)]
    pub fn render_frame(&mut self, egl_context: &EglContext, width: i32, height: i32) -> Result<(), String> {
        // timestamp-based throttle: skip if we rendered less than 16ms ago.
        // prevents the compositor from being flooded with frames faster than
        // the display refresh rate, which causes buffer overflows and crashes.
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_render_time).as_millis();
        if elapsed < MIN_FRAME_INTERVAL_MS {
            return Ok(());
        }

        // ask mpv if there is actually a new frame to render.
        // update() returns a bitflag; the Frame bit means "call render()".
        // without this check, we'd re-render stale frames on every event loop tick.
        let flags = self.render_context.update()
            .map_err(|e| format!("mpv context update error: {:?}", e))?;
        if (flags & mpv_render_update::Frame) == 0 {
            return Ok(());
        }

        // set the OpenGL viewport to match the full EGL surface dimensions.
        // without this, OpenGL uses whatever default viewport was set during
        // context creation, causing the video to render at the wrong size
        // (the "shrunk from both sides" bug).
        // The instruction at 0x00007FFCD04041CC referenced memory at 0x0000000000000000
        // OFCOURSE IT CRASHED THE WHOLE VM.
        unsafe {
            gl::Viewport(0, 0, width, height);
        }

        self.render_context.render::<()>(0, width, height, true)
            .map_err(|e| format!("mpv render error: {:?}", e))?;
            
        egl_context.swap_buffers()?;
        
        // inform libmpv that the buffer has been presented to the compositor.
        // this releases GL fence objects and internal resources associated with
        // the rendered frame. without this call, libmpv accumulates unreleased
        // GPU sync objects every frame, causing a memory leak that eventually
        // triggers the OOM killer (the "30 second crash" bug).
        self.render_context.report_swap();

        self.last_render_time = now;
        Ok(())
    }
}