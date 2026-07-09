// layer_shell::surface - wlr-layer-shell unified player loop
//
// orchestrates the layershellev wayland connection, the EGL context,
// and the MPV render pipeline in a single thread using calloop.

use layershellev::{
    Anchor, KeyboardInteractivity, Layer, LayerShellEvent, ReturnData, WindowState,
    NewLayerShellSettings, OutputOption,
    calloop::channel,
};
use libmpv2::{Mpv, render::{RenderParam, RenderParamApiType, OpenGLInitParams}};
use std::os::raw::c_void;

use crate::platform::linux::shared::types::MonitorInfo;
use crate::platform::linux::runner::config::MpvConfig;
use crate::platform::linux::shared::ipc;
use super::egl::EglContext;

#[derive(Debug)]
pub enum PlayerMessage {
    MpvRedrawRequested,
    IpcCommand(String),
}

pub fn run_player(monitor: &MonitorInfo, config: &MpvConfig, socket_path: String) -> Result<(), String> {
    println!("[layer_shell] creating background surface on monitor '{}'", monitor.name);

    // 1. Set up the layershellev state
    let ev: WindowState<()> = WindowState::new("colorwall-linux")
        .with_single(NewLayerShellSettings {
            size: None,
            layer: Layer::Background,
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
            exclusive_zone: Some(-1),
            margin: Some((0, 0, 0, 0)),
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::OutputName(monitor.name.clone()),
            events_transparent: true,
            namespace: Some("wallpaper".to_string()),
        })
        .build()
        .map_err(|e| format!("Failed to build WindowState: {:?}", e))?;

    // 2. Set up the IPC / MPV communication channel
    let (event_sender, event_receiver) = channel::channel::<PlayerMessage>();

    // Start IPC thread if socket path provided
    if !socket_path.is_empty() {
        let ipc_sender = event_sender.clone();
        std::thread::spawn(move || {
            ipc::start_listener(&socket_path, move |cmd| {
                let _ = ipc_sender.send(PlayerMessage::IpcCommand(cmd));
            });
        });
    }

    // Mutable state for the event loop
    let mut egl_context: Option<EglContext> = None;
    let mut mpv_player: Option<crate::platform::linux::runner::mpv::MpvPlayer> = None;
    let mut wl_egl_surface: Option<wayland_egl::WlEglSurface> = None;
    let mut current_size = (0, 0);

    let event_sender_clone = event_sender.clone();
    let config_clone = config.clone();

    // 3. Run the event loop
    ev.running_with_proxy(event_receiver, move |event, window_state, _index| {
        match event {
            LayerShellEvent::InitRequest => {
                let raw_display = window_state.get_connection().backend().display_ptr() as *mut c_void;
                match EglContext::new(raw_display) {
                    Ok(ctx) => {
                        egl_context = Some(ctx);
                        ReturnData::RequestBind
                    }
                    Err(e) => {
                        eprintln!("[layer_shell] EGL Init Error: {}", e);
                        ReturnData::RequestExit
                    }
                }
            }
            LayerShellEvent::BindProvide(_globals, _qh) => {
                ReturnData::RequestCompositor
            }
            LayerShellEvent::CompositorProvide(compositor, qh) => {
                for x in window_state.get_unit_iter() {
                    let region = compositor.create_region(qh, ());
                    region.add(0, 0, 0, 0); 
                    x.get_wlsurface().set_input_region(Some(&region));
                }
                ReturnData::None
            }
            LayerShellEvent::XdgInfoChanged(_) => {
                if let Some(unit) = window_state.get_unit_iter().next() {
                    let (width, height) = unit.get_size();
                    if width > 0 && height > 0 && current_size != (width, height) {
                        current_size = (width, height);
                        
                        if let Some(egl) = egl_context.as_mut() {
                            // 1. Create wayland-egl wrapper
                            let surface = wayland_egl::WlEglSurface::new(
                                unit.get_wlsurface().clone(),
                                width as i32,
                                height as i32,
                            ).expect("Failed to create WlEglSurface");
                            
                            // 2. Bind to EGL
                            egl.create_window_surface(surface.ptr() as *mut c_void)
                                .expect("Failed to create EGL window surface");
                            
                            // Keep it alive
                            wl_egl_surface = Some(surface);
                            
                            // 3. Init MPV now that EGL is ready
                            if mpv_player.is_none() {
                                mpv_player = Some(crate::platform::linux::runner::mpv::MpvPlayer::new(
                                    &config_clone,
                                    egl,
                                    event_sender_clone.clone(),
                                ).expect("Failed to init MPV"));
                            }
                        }
                    }
                }
                ReturnData::None
            }
            LayerShellEvent::UserEvent(PlayerMessage::MpvRedrawRequested) => {
                if let (Some(player), Some(egl)) = (mpv_player.as_mut(), egl_context.as_mut()) {
                    let (w, h) = current_size;
                    if let Err(e) = player.render_frame(egl, w as i32, h as i32) {
                        eprintln!("[layer_shell] mpv render error: {}", e);
                    }
                }
                ReturnData::None
            }
            LayerShellEvent::UserEvent(PlayerMessage::IpcCommand(cmd)) => {
                println!("[player] IPC command received: {}", cmd);
                if cmd == "STOP" {
                    ReturnData::RequestExit
                } else {
                    ReturnData::None
                }
            }
            _ => ReturnData::None,
        }
    })
    .map_err(|e| format!("Event loop error: {:?}", e))
}
