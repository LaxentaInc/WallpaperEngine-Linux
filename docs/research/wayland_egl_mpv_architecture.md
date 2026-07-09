# Wayland + EGL + MPV Architecture

## Introduction
Building a hardware-accelerated video wallpaper engine on Linux requires a specific initialization sequence to bridge Wayland's layer-shell, the EGL context, and MPV's render API. Since typical UI frameworks (like Tauri's webview or standard Winit) don't easily allow raw video to be painted behind the desktop icons on a hardware level, we use `libmpv`'s `RenderContext` directly tied to an EGL surface.

## The Initialization Pipeline
This is the required flow to successfully render MPV onto a Wayland layer-shell surface without X11 fallbacks or software rendering.

1. **Wayland Display Connection**
   - The application connects to the Wayland compositor.
   - We extract the raw `*mut wl_display` pointer.

2. **EGL Context Creation (Surfaceless)**
   - Before the window surface exists, we initialize EGL using the `wl_display`.
   - The context is made current with `EGL_NO_SURFACE`.
   - **Why?** MPV's OpenGL initialization requires a valid EGL context to load its functions (via `get_proc_address`), even before the window is sized.

3. **MPV Render API Initialization**
   - We initialize `libmpv2` with `vo=libmpv` (the render API).
   - We create a `RenderContext`, passing our EGL context and a custom `get_proc_address` function that resolves OpenGL symbols using EGL.

4. **Layer Surface Configuration**
   - We request a layer surface from the compositor (e.g., `Layer::Background`).
   - The compositor responds with an `XdgInfoChanged` or configure event, providing the assigned width and height.

5. **WlEglSurface Binding**
   - Once we have the dimensions, we wrap the `WlSurface` (using its `ObjectId`) into a `WlEglSurface`.
   - This `WlEglSurface` acts as the native window handle.
   - We tell EGL to create a window surface (`eglCreateWindowSurface`) targeting this handle.

6. **Render Loop Multiplexing**
   - We listen to MPV's `set_update_callback` which fires when a new video frame is ready.
   - We bounce this signal back to our main Wayland event loop (using a `calloop` channel).
   - On receive, we call `mpv.render()` and then `eglSwapBuffers()` to present the frame to Wayland.

## The `layershellev` Fork Justification
We originally attempted to use `layershellev` out-of-the-box. However, high-level Wayland wrappers hide the raw pointers needed for EGL. We forked `layershellev` into our workspace to expose:
1. `WindowState::get_connection()`: To get the raw `wl_display*` for `eglGetPlatformDisplay`.
2. `WindowStateUnit::get_wlsurface()`: To get the `WlSurface` proxy to create the `WlEglSurface`.
3. Re-exporting `Layer`, `Anchor`, etc. as public so our downstream code can configure the shell.
