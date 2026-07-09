tracing::instrument shall look into this tuff thing

some thinking: 
The wlr-layer-shell-unstable-v1 protocol dictates exactly how your client program talks to the Wayland compositor.zwlr_layer_shell_v1 (The Factory): This is the main global interface. Its only job is to let you take a standard, blank window surface (wl_surface) and turn it into a desktop layer surface via get_layer_surface.zwlr_layer_surface_v1 (The Configuration): This is where you configure your wallpaper behavior. The protocol dictates a strict sequence of events:You create the layer surface.You send configuration requests like set_layer(background), set_size(0, 0) (0 means let the compositor decide/fullscreen), and set_exclusive_zone(-1) (forces it to stretch under panels).The Wayland Constraint: You cannot attach a video buffer or tell libmpv to render yet. You must perform an initial wl_surface.commit without a buffer. The compositor will calculate the dimensions and fire a configure event back to you. You must reply with ack_configure, and only then can libmpv start rendering its frames.

saving sum context to confirm
 project isn't just starting a video player—your application is technically a custom graphics pipeline.[ Layershellev Event Loop ] ──(Pumps Tick)──> [ EGL Context ]
                                                    │
  ┌─────────────────────────────────────────────────┘
  ▼
[ mpv_render_context_render ] ──(Draw Frame)──> [ Raw Wayland Surface ]
Step 1: Initialize the EGL Display and SurfaceYou cannot draw to a raw Wayland surface without an active OpenGL canvas. Before you even touch libmpv, you must take the raw wl_display and wl_surface pointers from your modified layershellev crate and initialize a custom EGL window surface using Linux graphics bindings (like the khronos-egl or glutin crate).Step 2: Feed the GL Function Loader to MPVlibmpv needs to understand how to talk to your GPU on Linux. You have to pass mpv_render_context_alloc a callback function that handles OpenGL function pointer resolution (a standard GL loader helper).Step 3: Map the Frames inside the calloop Event loopInstead of libmpv driving the video frames on its own thread, your vendored layershellev engine (which runs on a calloop event mechanism) will orchestrate frame switches.libmpv will trigger a wake-up callback whenever a new video frame is ready (mpv_set_wakeup_callback).Your event loop intercepts this wakeup, activates your EGL context, and evaluates mpv_render_context_render().You immediately execute a eglSwapBuffers call to push that video chunk natively onto the wl_surface backdrop.Immediate Action Items to Add to Your PlanVerify if mpv-rs supports the Render API: Standard Rust bindings for mpv sometimes only support command-line bindings (--wid). Ensure the wrapper or raw FFI bindings you use expose the low-level mpv_render_context C functions.Add an explicit Pause-State Handler: Because this is a wallpaper engine, your background loops will destroy battery life and GPU performance if left unchecked. Your code needs to check for desktop state flags or visibility hints so it can aggressively call mpv.set_property("pause", true) whenever a user launches a fullscreen web browser or game over your desktop backdrop.



Params for LayerShellSurface::create()
surface: The underlying, unmapped base Wayland surface.output: The specific monitor (wl_output) where the layer will display (or NULL/None for the compositor default).layer: The absolute Z-depth index of the stack.namespace: A descriptive string identifier for the compositor to apply specific rules (e.g., "panel", "wallpaper").   

KEEPING TRACK in case i forget;
pub use wayland_client; pub use wayland_backend; (line 217-218)	Our code needs to use WlSurface, WlDisplay, Connection etc. If we add wayland-client as a separate dependency in our Cargo.toml, Cargo might resolve a different minor version (e.g. 0.31.8 vs layershellev's 0.31.7). Same crate, different versions = incompatible types. By re-exporting from layershellev, we guarantee we're using the exact same copy.

get_connection() on WindowState (line 1568)	EGL needs the raw C pointer wl_display* to call eglGetPlatformDisplay(). The WlDisplay proxy is a Rust wrapper - it's NOT the C pointer. The actual C pointer lives inside Connection's backend (connection.backend().display_ptr()).

get_display(), get_wl_surface() on WindowStateUnit (line 555-560)	These expose the Wayland proxy objects. We need wl_surface to create a wl_egl_window which becomes our EGL render target.