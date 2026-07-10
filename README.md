# ColorWall Linux

ColorWall Linux is a native Linux Wallpaper Engine built with Rust, Tauri, and `libmpv`. 

It is currently heavily work-in-progress (WIP). The architecture relies on directly interacting with Wayland compositors to render hardware-accelerated video backgrounds underneath desktop icons using the `wlr-layer-shell` protocol.

## Architecture & Tech Stack

- **Frontend:** React + TypeScript (Vite) wrapped in Tauri.
- **Backend IPC/Engine:** Rust (Tauri commands).
- **Sidecar Video Player:** A standalone Rust binary (`cl-video-player`) spawned by the Tauri backend.
- **Wayland / EGL:** Uses a custom fork of `layershellev` to manage the layer shell, exposing raw `wl_display` and `wl_surface` pointers.
- **Rendering:** Uses `khronos-egl` for FFI OpenGL context creation, which is fed directly into `libmpv`'s render API for hardware video decoding.
- **Event Loop:** Merges Wayland compositor events (configure/resizes) and `libmpv` redraw ticks into a single `calloop` event loop to avoid multithreading collisions on the EGL context.

## Current State

- ✅ Basic Tauri IPC bridge and Sidecar spawning.
- ✅ `layershellev` fork integrated and Wayland `wlr-layer-shell` background surfaces rendering on KWin (KDE Plasma).
- ✅ EGL OpenGL surface extracted from `WlSurface` (via `wayland-egl`).
- ✅ `libmpv` initialization and context binding.
- 🚧 **IPC Socket Listener:** Passing pause/play commands from the Tauri UI into the MPV render loop via Unix Domain Sockets (currently being implemented).
- 🚧 **X11 / GNOME Fallback:** `mutter.rs` stub is currently empty. The engine only runs on compositors supporting `wlr-layer-shell` right now.

## Running

This project **cannot** be compiled or run on Windows. It contains deep native Linux C bindings and relies on Wayland protocols.

You must build and run it on a Linux machine or VM.
```bash
# Start the Tauri dev server
pnpm tauri dev
```
