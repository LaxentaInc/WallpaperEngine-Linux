// wayland::surface - wlr-layer-shell unified player loop
//
// orchestrates the layershellev wayland connection, the EGL context,
// and the MPV render pipeline in a single thread using calloop.
//
// this file is the primary diagnostic surface for debugging compositor
// interactions. every lifecycle event, layer configuration, and render
// state is logged with full detail so we never have to guess what the
// compositor is doing.

use layershellev::{
    Anchor, KeyboardInteractivity, Layer, LayerShellEvent, ReturnData, WindowState,
    calloop::channel,
};
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

/// logs the full wayland layer-shell configuration that we are requesting
/// from the compositor. this is the equivalent of logging WorkerW layers
/// on windows - full transparency into what shell properties we are setting.
fn log_layer_config(monitor: &MonitorInfo) {
    tracing::info!("════════════════════════════════════════════════════════════");
    tracing::info!("  WAYLAND LAYER-SHELL CONFIGURATION");
    tracing::info!("════════════════════════════════════════════════════════════");
    tracing::info!("  target monitor:     {} (id: {})", monitor.name, monitor.id);
    tracing::info!("  monitor resolution: {}x{}", monitor.width, monitor.height);
    tracing::info!("  monitor scale:      {}", monitor.scale);
    tracing::info!("  monitor position:   ({}, {})", monitor.x, monitor.y);
    tracing::info!("  monitor primary:    {}", monitor.primary);
    tracing::info!("  ──────────────────────────────────────────────────────────");
    tracing::info!("  layer:              Background (zwlr_layer_shell_v1)");
    tracing::info!("  anchor:             Top | Bottom | Left | Right (fullscreen)");
    tracing::info!("  exclusive_zone:     -1 (do not reserve screen space)");
    tracing::info!("  keyboard:           None (no keyboard grab)");
    tracing::info!("  input_region:       empty (all input passes through)");
    tracing::info!("  use_display_handle: true (we manage EGL rendering ourselves)");
    tracing::info!("  events_transparent: true (pointer events fall through)");
    tracing::info!("════════════════════════════════════════════════════════════");
}

#[tracing::instrument(skip_all)]
pub fn run_player(monitor: &MonitorInfo, config: &MpvConfig, socket_path: String) -> Result<(), String> {
    tracing::info!("creating background surface on monitor '{}'", monitor.name);

    // log the full layer-shell configuration for diagnostics
    log_layer_config(monitor);

    // set up the layershellev state
    // with_xdg_output_name targets the specific monitor by its xdg output name
    // (e.g. "HDMI-A-1", "eDP-1") which MonitorInfo.name already provides.
    // with_events_transparent makes the surface ignore all pointer/keyboard input,
    // so clicks fall through to desktop icons on the layer above.
    let ev: WindowState<()> = WindowState::new("colorwall-linux")
        .with_layer(Layer::Background)
        .with_anchor(Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top)
        .with_exclusive_zone(-1)
        .with_keyboard_interacivity(KeyboardInteractivity::None)
        // temporarily disabled: monitor.name is a dummy string ("primary") which 
        // causes Wayland to reject the layer. falling back to default monitor.
        // .with_xdg_output_name(monitor.name.clone())
        .with_events_transparent(true)
        .with_use_display_handle(true)
        .build()
        .map_err(|e| format!("Failed to build WindowState: {:?}", e))?;

    tracing::info!("[layer-shell] WindowState built successfully, entering event loop");

    // Set up the IPC / MPV communication channel
    let (event_sender, event_receiver) = channel::channel::<PlayerMessage>();

    // Start IPC thread if socket path provided
    if !socket_path.is_empty() {
        let ipc_sender = event_sender.clone();
        std::thread::spawn(move || {
            ipc::start_listener(&socket_path, move |cmd| {
                let _ = ipc_sender.send(PlayerMessage::IpcCommand(cmd.to_string()));
            });
        });
    }

    // Mutable state for the event loop
    let mut egl_context: Option<EglContext> = None;
    let mut mpv_player: Option<crate::platform::linux::runner::mpv::MpvPlayer> = None;
    let mut _wl_egl_surface: Option<wayland_egl::WlEglSurface> = None;
    let mut current_size = (0u32, 0u32);
    let mut frame_count: u64 = 0;

    let event_sender_clone = event_sender.clone();
    let config_clone = config.clone();

    // Run the event loop
    ev.running_with_proxy(event_receiver, move |event, window_state, _index| {
        match event {
            LayerShellEvent::InitRequest => {
                tracing::info!("[lifecycle] InitRequest - extracting raw wl_display pointer for EGL");
                let raw_display = window_state.get_connection().backend().display_ptr() as *mut c_void;
                tracing::info!("[lifecycle] raw wl_display pointer: {:?}", raw_display);

                match EglContext::new(raw_display) {
                    Ok(ctx) => {
                        tracing::info!("[lifecycle] EGL context created successfully (surfaceless)");
                        tracing::info!("[lifecycle] EGL display: {:?}, context: {:?}", ctx.display, ctx.context);
                        tracing::info!("[lifecycle] GL function pointers loaded via eglGetProcAddress");
                        egl_context = Some(ctx);
                        tracing::info!("[lifecycle] -> returning RequestBind to proceed to BindProvide");
                        ReturnData::RequestBind
                    }
                    Err(e) => {
                        tracing::error!("[lifecycle] EGL Init FAILED: {}", e);
                        tracing::error!("[lifecycle] -> returning RequestExit, cannot render without EGL");
                        ReturnData::RequestExit
                    }
                }
            }
            LayerShellEvent::BindProvide(_globals, _qh) => {
                tracing::info!("[lifecycle] BindProvide - wayland globals are now available");
                tracing::info!("[lifecycle] -> returning RequestCompositor to get wl_compositor access");
                ReturnData::RequestCompositor
            }
            LayerShellEvent::CompositorProvide(compositor, qh) => {
                tracing::info!("[lifecycle] CompositorProvide - wl_compositor access granted");

                // log all surface units that layershellev created for us
                let unit_count = window_state.get_unit_iter().count();
                tracing::info!("[layer-shell] total surface units created: {}", unit_count);

                for (i, x) in window_state.get_unit_iter().enumerate() {
                    let (w, h) = x.get_size();
                    tracing::info!("[layer-shell] unit[{}]: size={}x{}, wl_surface={:?}",
                        i, w, h, x.get_wlsurface());

                    // create an empty input region so all pointer/keyboard events
                    // pass through to whatever is below us (desktop icons, etc).
                    let region = compositor.create_region(qh, ());
                    region.add(0, 0, 0, 0); 
                    x.get_wlsurface().set_input_region(Some(&region));
                    x.get_wlsurface().commit();
                    tracing::info!("[layer-shell] unit[{}]: empty input region set + surface committed", i);
                }
                ReturnData::None
            }
            LayerShellEvent::XdgInfoChanged(change_type) => {
                // log xdg output changes - these tell us about monitor hotplug,
                // resolution changes, name changes, etc. critical for multi-monitor.
                tracing::info!("[xdg-output] info changed: {:?}", change_type);
                if let Some(unit) = window_state.get_unit_iter().next() {
                    let (w, h) = unit.get_size();
                    tracing::info!("[xdg-output] current unit size after change: {}x{}", w, h);
                }
                ReturnData::None
            }
            LayerShellEvent::RequestMessages(&layershellev::DispatchMessage::RequestRefresh { width, height, .. }) => {
                if let Some(unit) = window_state.get_unit_iter().next() {
                    if width > 0 && height > 0 && current_size != (width, height) {
                        tracing::info!("════════════════════════════════════════════════════════════");
                        tracing::info!("[configure] compositor assigned surface size: {}x{}", width, height);
                        tracing::info!("[configure] previous size: {}x{}", current_size.0, current_size.1);
                        tracing::info!("════════════════════════════════════════════════════════════");
                        current_size = (width, height);
                        
                        if let Some(egl) = egl_context.as_mut() {
                            // create the wayland-egl wrapper around the wl_surface.
                            // WlEglSurface::new calls wl_egl_window_create under the hood,
                            // which gives EGL a native window handle to render into.
                            use layershellev::wayland_client::Proxy;
                            tracing::info!("[egl] creating WlEglSurface (wl_egl_window_create) for {}x{}", width, height);

                            let surface = wayland_egl::WlEglSurface::new(
                                unit.get_wlsurface().id(),
                                width as i32,
                                height as i32,
                            ).expect("Failed to create WlEglSurface");
                            
                            tracing::info!("[egl] WlEglSurface created, native handle: {:?}", surface.ptr());

                            // Bind to EGL
                            egl.create_window_surface(surface.ptr() as *mut c_void)
                                .expect("Failed to create EGL window surface");
                            
                            tracing::info!("[egl] EGL window surface bound and made current");

                            // Keep it alive - if this drops, the wl_egl_window is destroyed
                            // and EGL loses its rendering target
                            _wl_egl_surface = Some(surface);
                            
                            // Init MPV now that EGL is ready
                            if mpv_player.is_none() {
                                tracing::info!("[mpv] initializing mpv player with EGL context...");
                                match crate::platform::linux::runner::mpv::MpvPlayer::new(
                                    &config_clone,
                                    egl,
                                    event_sender_clone.clone(),
                                ) {
                                    Ok(player) => {
                                        tracing::info!("[mpv] player initialized successfully, video loaded");
                                        tracing::info!("[mpv] render loop is now active, waiting for MpvRedrawRequested events");
                                        mpv_player = Some(player);
                                    }
                                    Err(e) => {
                                        tracing::error!("[mpv] FAILED to initialize player: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                ReturnData::None
            }
            LayerShellEvent::UserEvent(PlayerMessage::MpvRedrawRequested) => {
                if let (Some(player), Some(egl)) = (mpv_player.as_mut(), egl_context.as_mut()) {
                    let (w, h) = current_size;
                    match player.render_frame(egl, w as i32, h as i32) {
                        Ok(()) => {
                            frame_count += 1;
                            // log every 300 frames (~5 seconds at 60fps) to show the
                            // render loop is alive without spamming stdout
                            if frame_count % 300 == 0 {
                                tracing::info!("[render] frame #{} rendered at {}x{}", frame_count, w, h);
                            }
                        }
                        Err(e) => {
                            tracing::error!("[render] frame #{} FAILED: {}", frame_count, e);
                        }
                    }
                }
                ReturnData::None
            }
            LayerShellEvent::UserEvent(PlayerMessage::IpcCommand(cmd)) => {
                tracing::info!("[ipc] command received: {}", cmd);
                if cmd == "STOP" {
                    tracing::info!("[ipc] STOP command - requesting exit from event loop");
                    ReturnData::RequestExit
                } else {
                    ReturnData::None
                }
            }
            LayerShellEvent::NormalDispatch => {
                // normal tick of the event loop - no events pending.
                // intentionally silent to avoid log spam.
                ReturnData::None
            }
            _ => ReturnData::None,
        }
    })
    .map_err(|e| format!("Event loop error: {:?}", e))
}
