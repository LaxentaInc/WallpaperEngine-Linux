# ColorWall Linux Port: Error Log & Fixes

This document serves as a historical log of every technical hurdle, compilation error, and architectural mismatch we encountered while bridging `layershellev`, `wayland-egl`, `khronos-egl`, and `libmpv2` in Rust. It documents the exact root causes and the specific solutions applied, so we never lose context on why certain code exists.

---

## 1. `wayland-client` Type Mismatch
**Error:** `mismatched types: expected struct wayland_client::protocol::wl_surface::WlSurface (from wayland-client 0.31), found struct WlSurface (from layershellev's internal wayland-client 0.31)`
**File:** `Cargo.toml`, `src-tauri/layershellev/src/lib.rs`
**Root Cause:** The `ColorWall` crate had a direct dependency on `wayland-client = "0.31"` in its `Cargo.toml`. At the same time, `layershellev` internally depends on `wayland-client = "0.31"`. Cargo resolved these as two distinct crates in the dependency tree, causing Rust to treat them as separate, incompatible types even though they were identical.
**Fix:** Removed the direct `wayland-client` dependency from `ColorWall`'s `Cargo.toml`. Instead, we updated our local fork of `layershellev/src/lib.rs` to publicly re-export its internal `wayland-client` and `wayland-backend` crates (`pub use wayland_client; pub use wayland_backend;`). We now import Wayland types directly via `layershellev::wayland_client::*`.

---

## 2. Unsafe FFI Calls to `khronos-egl`
**Error:** `call to unsafe function is unsafe and requires unsafe function or block`
**File:** `src-tauri/src/platform/linux/layer_shell/egl.rs`
**Root Cause:** Calling C-FFI functions like `eglGetPlatformDisplay`, `eglInitialize`, and `eglCreateWindowSurface` bypasses Rust's safety guarantees because they involve raw C pointers (`*mut c_void`).
**Fix:** Wrapped the specific FFI calls in `unsafe {}` blocks inside `egl.rs`. We ensure safety manually by verifying the pointers passed from `wayland-backend` and `wayland-egl` are non-null and valid for the lifetime of the EGL context.

---

## 3. `RenderContext` and `Mpv` Lifetime Collision (E0502)
**Error:** `cannot borrow *mpv_ref as mutable because it is also borrowed as immutable` inside the `MpvPlayer` struct initialization.
**File:** `src-tauri/src/platform/linux/runner/mpv.rs`
**Root Cause:** The `libmpv2::render::RenderContext<'a>` requires a borrow of the `Mpv` instance for its entire lifetime. We attempted to store both `mpv: &'static mut Mpv` and `render_context: RenderContext<'static>` in the same struct. However, `create_render_context` immutably borrows the `Mpv` instance. Rust forbids holding both a mutable reference and an active immutable reference simultaneously (a self-referencing struct borrow conflict).
**Fix:** 
1. Used `Box::leak(Box::new(mpv))` to force the `Mpv` instance to live for `'static` (since the player lives for the duration of the app).
2. Casted the leaked reference to an immutable reference: `let mpv_ref: &'static Mpv = Box::leak(Box::new(mpv));`.
3. Stored `mpv: &'static Mpv` in the struct instead of a mutable reference. Since `mpv.command()` only requires `&self` (immutable), we retain full control without fighting the borrow checker.

---

## 4. `libmpv2` Render API Signature Change
**Error:** `this method takes 4 arguments but 1 argument was supplied` for `render_context.render()`.
**File:** `src-tauri/src/platform/linux/runner/mpv.rs`
**Root Cause:** In older versions of `libmpv2`, `.render()` took a `Vec<RenderParam>` (e.g., `RenderOpenglFbo`). In version `6.0.0`, the method signature changed to directly accept the FBO ID, width, height, and flip flag: `pub fn render<GLContext: 'static>(&mut self, fbo: i32, w: i32, h: i32, flip_y: bool)`.
**Fix:** Removed the `RenderParam::OpenglFbo` vector and updated the call to `self.render_context.render::<()>(0, width, height, true)`.

---

## 5. Private Types in `layershellev` (E0603)
**Error:** `enum KeyboardInteractivity is private`, `struct Anchor is private`, `enum Layer is private`.
**File:** `src-tauri/layershellev/src/lib.rs`
**Root Cause:** `layershellev` uses `wayland_protocols_wlr` internally but didn't publicly re-export the enums needed to configure the window state.
**Fix:** Modified our local fork (`layershellev/src/lib.rs`) to change `use wayland_protocols_wlr::...` to `pub use wayland_protocols_wlr::...`.

---

## 6. Wrong `with_output_option` API
**Error:** `no method named with_output_option found for struct WindowState<T>`
**File:** `src-tauri/src/platform/linux/layer_shell/surface.rs`
**Root Cause:** A hallucinated/misremembered API method for targeting a specific monitor was written in `surface.rs`. The `layershellev` crate does not have a `OutputOption` enum on its builder.
**Fix:** Grepped the `layershellev` source code and discovered the correct method is `with_xdg_output_name(String)`. Replaced the fake method with `.with_xdg_output_name(monitor.name.clone())`.

---

## 7. `WlEglSurface::new` Expected `ObjectId` (E0308)
**Error:** `mismatched types: expected ObjectId, found &WlSurface` when calling `wayland_egl::WlEglSurface::new`.
**File:** `src-tauri/src/platform/linux/layer_shell/surface.rs`
**Root Cause:** We passed the `WlSurface` proxy reference directly to the EGL surface creator. However, in `wayland-egl` version `0.31`, the constructor explicitly expects the raw `ObjectId` of the Wayland surface, not the proxy object itself.
**Fix:** Imported the `Proxy` trait from our re-exported `layershellev::wayland_client::Proxy` and called `.id()` on the surface: `unit.get_wlsurface().id()`.

---

## 8. Duplicate Crate Versions in Dependency Tree (E0308)
**Error:** `expected wayland_backend::sys::client::ObjectId, found ObjectId. note: there are multiple different versions of crate wayland_backend in the dependency graph`
**File:** `src-tauri/Cargo.toml`
**Root Cause:** We fixed the previous error by calling `.id()`, but `Cargo` ended up pulling two completely different major versions of `wayland-backend` (`0.2.0` and `0.3.15`). `layershellev` uses `wayland-client 0.31` which pairs with `wayland-backend 0.3`. However, we explicitly depended on `wayland-egl = "0.31"`, and it turns out the `0.31` version of `wayland-egl` relies on the older `wayland-backend 0.2.0`.
**Fix:** Bumped `wayland-egl` from `"0.31"` to `"0.32"` in `Cargo.toml`. The `0.32` version correctly aligns with `wayland-backend 0.3.x`, fully syncing our dependency tree and unifying the `ObjectId` types.

---

## 9. Unused wl_egl_surface warning
**Error:** `unused variable: wl_egl_surface`
**File:** `src-tauri/src/platform/linux/layer_shell/surface.rs`
**Root Cause:** Assigned `wl_egl_surface = Some(surface)` but never read from it again. This is intentional because we only need to store it to prevent Rust from dropping it and destroying the EGL window.
**Fix:** Explicitly told the compiler the unused status is intentional by prefixing the variable with an underscore: `let mut _wl_egl_surface`.

---

## 10. Unused Result warning
**Error:** `unused Result that must be used`
**File:** `src-tauri/src/platform/linux/runner/mpv.rs`
**Root Cause:** The method `self.render_context.update()` returns a `Result<(), Error>` which we were silently dropping. Rust expects all `Result` variants to be explicitly handled or discarded.
**Fix:** Chained a `.map_err()` to map the underlying error to our custom `String` error type and propagated it using the `?` operator.

---

## 11. `Connection not initialized yet` panic in layershellev
**Error:** `thread 'main' panicked at layershellev/layershellev/src/lib.rs:1573:34: Connection not initialized yet`
**File:** `src-tauri/layershellev/layershellev/src/lib.rs`
**Root Cause:** In `running_with_proxy_option()`, right before the event loop dispatches `InitRequest`, the library calls `self.connection.take().unwrap()` (line 2453). `.take()` moves the `Connection` out of the `WindowState` struct, setting the `Option` to `None`. When our `InitRequest` handler then called `window_state.get_connection()`, it panicked because the connection had been consumed. Since Wayland `Connection` objects are `Arc`-wrapped and cheap to clone (layershellev itself clones them internally for `CursorUpdateContext`), using `.clone()` instead of `.take()` is safe.
**Fix:** Changed `self.connection.take().unwrap()` to `self.connection.clone().unwrap()` in `layershellev/src/lib.rs:2453`.

---

## 12. `You cannot return this one` panic (missing use_display_handle)
**Error:** `thread 'main' panicked at layershellev/layershellev/src/lib.rs:3031:29: You cannot return this one`
**File:** `src-tauri/src/platform/linux/wayland/layer_shell/surface.rs`
**Root Cause:** When `use_display_handle` is `false` (the default), `layershellev` expects each surface unit to provide a software `WlBuffer` via the `RequestBuffer` event. Since we use hardware EGL rendering and our catch-all handler returns `ReturnData::None` instead of `ReturnData::WlBuffer(...)`, the library panics. Setting `use_display_handle = true` tells layershellev to skip the software buffer path entirely, deferring all rendering control to the caller.
**Fix:** Added `.with_use_display_handle(true)` to the `WindowState` builder chain in `surface.rs`.

---

## 13. Missing `report_swap()` causing 30-second process death
**Error:** cl-video-player process dies after ~30 seconds of rendering, no error message, likely OOM-killed.
**File:** `src-tauri/src/platform/linux/runner/mpv.rs`
**Root Cause:** After rendering a frame and calling `eglSwapBuffers()`, we were calling `render_context.update()` instead of `render_context.report_swap()`. The `update()` method checks if a new frame is available (a pre-render query), while `report_swap()` informs libmpv that the buffer was actually presented to the compositor. Without `report_swap()`, libmpv accumulates GL fence objects and internal sync resources for every rendered frame, causing a memory leak. Additionally, we had no frame throttling, so MPV's update callback fired at unlimited speed, flooding the compositor. Combined, these caused OOM-kill or compositor-initiated client termination after ~30 seconds.
**Fix:** (1) Moved `update()` to be a pre-check before `render()`, only rendering when the `Frame` flag is set. (2) Replaced the post-render `update()` call with `report_swap()`. (3) Added timestamp-based frame throttling (16ms minimum interval, ~60fps cap). (4) Added `glViewport()` call before render to fix video not filling the monitor surface. (5) Added `gl::load_with()` in `egl.rs` to load GL function pointers via EGL.
