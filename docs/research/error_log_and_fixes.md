# ColorWall Linux Port: Error Log & Fixes

This document serves as a historical log of every technical hurdle, compilation error, and architectural mismatch we encountered while bridging `layershellev`, `wayland-egl`, `khronos-egl`, and `libmpv2` in Rust. It documents the exact root causes and the specific solutions applied, so we never lose context on why certain code exists.

---

## 1. `wayland-client` Type Mismatch
**Error:** `mismatched types: expected struct wayland_client::protocol::wl_surface::WlSurface (from wayland-client 0.31), found struct WlSurface (from layershellev's internal wayland-client 0.31)`
**Root Cause:** The `ColorWall` crate had a direct dependency on `wayland-client = "0.31"` in its `Cargo.toml`. At the same time, `layershellev` internally depends on `wayland-client = "0.31"`. Cargo resolved these as two distinct crates in the dependency tree, causing Rust to treat them as separate, incompatible types even though they were identical.
**Fix:** Removed the direct `wayland-client` dependency from `ColorWall`'s `Cargo.toml`. Instead, we updated our local fork of `layershellev/src/lib.rs` to publicly re-export its internal `wayland-client` and `wayland-backend` crates (`pub use wayland_client; pub use wayland_backend;`). We now import Wayland types directly via `layershellev::wayland_client::*`.

---

## 2. Unsafe FFI Calls to `khronos-egl`
**Error:** `call to unsafe function is unsafe and requires unsafe function or block`
**Root Cause:** Calling C-FFI functions like `eglGetPlatformDisplay`, `eglInitialize`, and `eglCreateWindowSurface` bypasses Rust's safety guarantees because they involve raw C pointers (`*mut c_void`).
**Fix:** Wrapped the specific FFI calls in `unsafe {}` blocks inside `egl.rs`. We ensure safety manually by verifying the pointers passed from `wayland-backend` and `wayland-egl` are non-null and valid for the lifetime of the EGL context.

---

## 3. `RenderContext` and `Mpv` Lifetime Collision (E0502)
**Error:** `cannot borrow *mpv_ref as mutable because it is also borrowed as immutable` inside the `MpvPlayer` struct initialization.
**Root Cause:** The `libmpv2::render::RenderContext<'a>` requires a borrow of the `Mpv` instance for its entire lifetime. We attempted to store both `mpv: &'static mut Mpv` and `render_context: RenderContext<'static>` in the same struct. However, `create_render_context` immutably borrows the `Mpv` instance. Rust forbids holding both a mutable reference and an active immutable reference simultaneously (a self-referencing struct borrow conflict).
**Fix:** 
1. Used `Box::leak(Box::new(mpv))` to force the `Mpv` instance to live for `'static` (since the player lives for the duration of the app).
2. Casted the leaked reference to an immutable reference: `let mpv_ref: &'static Mpv = Box::leak(Box::new(mpv));`.
3. Stored `mpv: &'static Mpv` in the struct instead of a mutable reference. Since `mpv.command()` only requires `&self` (immutable), we retain full control without fighting the borrow checker.

---

## 4. `libmpv2` Render API Signature Change
**Error:** `this method takes 4 arguments but 1 argument was supplied` for `render_context.render()`.
**Root Cause:** In older versions of `libmpv2`, `.render()` took a `Vec<RenderParam>` (e.g., `RenderOpenglFbo`). In version `6.0.0`, the method signature changed to directly accept the FBO ID, width, height, and flip flag: `pub fn render<GLContext: 'static>(&mut self, fbo: i32, w: i32, h: i32, flip_y: bool)`.
**Fix:** Removed the `RenderParam::OpenglFbo` vector and updated the call to `self.render_context.render::<()>(0, width, height, true)`.

---

## 5. Private Types in `layershellev` (E0603)
**Error:** `enum KeyboardInteractivity is private`, `struct Anchor is private`, `enum Layer is private`.
**Root Cause:** `layershellev` uses `wayland_protocols_wlr` internally but didn't publicly re-export the enums needed to configure the window state.
**Fix:** Modified our local fork (`layershellev/src/lib.rs`) to change `use wayland_protocols_wlr::...` to `pub use wayland_protocols_wlr::...`.

---

## 6. Wrong `with_output_option` API
**Error:** `no method named with_output_option found for struct WindowState<T>`
**Root Cause:** A hallucinated/misremembered API method for targeting a specific monitor was written in `surface.rs`. The `layershellev` crate does not have a `OutputOption` enum on its builder.
**Fix:** Grepped the `layershellev` source code and discovered the correct method is `with_xdg_output_name(String)`. Replaced the fake method with `.with_xdg_output_name(monitor.name.clone())`.

---

## 7. `WlEglSurface::new` Expected `ObjectId` (E0308)
**Error:** `mismatched types: expected ObjectId, found &WlSurface` when calling `wayland_egl::WlEglSurface::new`.
**Root Cause:** We passed the `WlSurface` proxy reference directly to the EGL surface creator. However, in `wayland-egl` version `0.31`, the constructor explicitly expects the raw `ObjectId` of the Wayland surface, not the proxy object itself.
**Fix:** Imported the `Proxy` trait from our re-exported `layershellev::wayland_client::Proxy` and called `.id()` on the surface: `unit.get_wlsurface().id()`.

---

## 8. Duplicate Crate Versions in Dependency Tree (E0308)
**Error:** `expected wayland_backend::sys::client::ObjectId, found ObjectId. note: there are multiple different versions of crate wayland_backend in the dependency graph`
**Root Cause:** We fixed the previous error by calling `.id()`, but `Cargo` ended up pulling two completely different major versions of `wayland-backend` (`0.2.0` and `0.3.15`). `layershellev` uses `wayland-client 0.31` which pairs with `wayland-backend 0.3`. However, we explicitly depended on `wayland-egl = "0.31"`, and it turns out the `0.31` version of `wayland-egl` relies on the older `wayland-backend 0.2.0`.
**Fix:** Bumped `wayland-egl` from `"0.31"` to `"0.32"` in `Cargo.toml`. The `0.32` version correctly aligns with `wayland-backend 0.3.x`, fully syncing our dependency tree and unifying the `ObjectId` types.
