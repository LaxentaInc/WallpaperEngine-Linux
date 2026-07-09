# ColorWall Linux Port Roadmap

## Completed Milestones
- [x] **Setup Repository & Submodules**: Cloned `layershellev` fork into `src-tauri` workspace for custom tweaks.
- [x] **Wayland EGL Integration (`egl.rs`)**: Implemented EGL initialization logic over the raw Wayland Display pointer using `khronos-egl` dynamically.
- [x] **MPV Integration (`mpv.rs`)**: Wired `libmpv2`'s `RenderContext` to use our custom EGL functions to render video frames directly onto the screen.
- [x] **Event Loop Unification (`surface.rs`)**: Combined the `layershellev` GUI loop and the MPV redraw signals into a single `calloop` loop to prevent multi-threading sync issues.

## Current Status
- Resolving the final Rust compilation issues on the Linux VM.
- We have correctly extracted the `ObjectId` from `WlSurface` to feed into `wayland_egl::WlEglSurface`.

## Next Steps (What's left?)
1. **Compilation Success**: Wait for confirmation from the Linux VM that the Rust `cargo build` succeeds cleanly.
2. **Runtime Verification**: Verify that the application launches, connects to Wayland, configures the `LayerShell` background surface, and renders MPV content without segfaulting on `eglSwapBuffers`.
3. **IPC Channel**: Connect the standard cross-platform IPC socket to send events like "Pause", "Play", "Change Volume", and "Change Video" into the `surface.rs` event loop.
4. **Mutter/X11 Fallback (`mutter.rs`)**: If the user is on X11 or a compositor that doesn't support layer-shell (like standard GNOME without extensions), we need a fallback rendering mode. Currently, this is heavily stubbed.

## Outstanding Questions
- Does `layershellev`'s `.with_events_transparent(true)` correctly allow mouse clicks to pass through to the standard desktop? We must test this in runtime.
- Do we need an audio stream? (Currently forced disabled via `mpv` properties).
