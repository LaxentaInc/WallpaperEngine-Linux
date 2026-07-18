// video::mpv - libmpv video playback engine
//
// pure mpv initialization and control. this module receives an EGL context
// and sets up the libmpv render API to paint frames via OpenGL onto the Wayland surface.

use super::config::MpvConfig;
use crate::platform::linux::wayland::egl::EglContext;
use layershellev::calloop::channel::Sender;
use crate::platform::linux::wayland::surface::PlayerMessage;
use libmpv2::{
    Mpv,
    render::{OpenGLInitParams, RenderParam, RenderParamApiType},
};
use std::ffi::c_void;

/// helper to unwrap the context pointer and pass to our EGL proc loader
fn mpv_get_proc_address(ctx: &*const c_void, name: &str) -> *mut c_void {
    let egl = unsafe { &*(*ctx as *const EglContext) };
    EglContext::get_proc_address(egl, name)
}

pub struct MpvPlayer {
    pub mpv: &'static Mpv,
    pub render_context: libmpv2::render::RenderContext<'static>,
}

impl MpvPlayer {
    pub fn new(
        config: &MpvConfig,
        egl_context: &EglContext,
        event_sender: Sender<PlayerMessage>,
    ) -> Result<Self, String> {
        println!("[mpv] initializing libmpv context with EGL render API...");
        
        let mpv = Mpv::with_initializer(|init| {
            init.set_property("vo", "libmpv").unwrap();
            init.set_property("hwdec", "auto-safe").unwrap();
            
            init.set_property("profile", "fast").unwrap();
            init.set_property("vd-lavc-fast", "yes").unwrap();
            init.set_property("vd-lavc-skiploopfilter", "all").unwrap();
            init.set_property("osc", "no").unwrap();
            init.set_property("window-dragging", "no").unwrap();
            init.set_property("input-default-bindings", "no").unwrap();
            init.set_property("audio", "no").unwrap();
            init.set_property("border", "no").unwrap();
            // we will Loop by default
            // TODO: KEEP THIS BUT REMOVE THIS FROM SETTINGS. SO No more config reads to set the property flag. 
            if config.loop_playback {
                init.set_property("loop-file", "inf").unwrap();
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

        println!("[mpv] loading video: {}", config.video_path);
        mpv_ref.command("loadfile", &[&config.video_path])
            .map_err(|e| format!("failed to load video: {}", e))?;

        Ok(Self {
            mpv: mpv_ref,
            render_context,
        })
    }

    /// Instructs MPV to render the current frame and swaps EGL buffers
    pub fn render_frame(&mut self, egl_context: &EglContext, width: i32, height: i32) -> Result<(), String> {
        self.render_context.render::<()>(0, width, height, true)
            .map_err(|e| format!("mpv render error: {:?}", e))?;
            
        egl_context.swap_buffers()?;
        
        self.render_context.update()
            .map_err(|e| format!("mpv context update error: {:?}", e))?;
        Ok(())
    }
}