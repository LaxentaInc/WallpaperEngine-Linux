# Contributing to ColorWall Linux

As this project relies heavily on low-level Linux C FFI (Foreign Function Interface), Wayland protocol extensions, and EGL context management, development requires strict adherence to our workflow to prevent undocumented memory unsafety or architectural drift.

## 1. Development Environment

**This project cannot be compiled on Windows or macOS.** 
If your IDE is on Windows, you must sync your files to a Linux Virtual Machine or a remote Linux host to compile the `src-tauri` workspace.

Do **NOT** run `cargo build` locally on a non-Linux machine.

If you add any native C dependencies (e.g., `libglib2.0-dev`), ensure they are installed on the target Linux build environment.

## 2. Documentation & Error Logging (CRITICAL)

When you encounter a compiler error, a segfault, or a Wayland protocol violation, you **MUST** document the fix in `docs/research/error_log_and_fixes.md`.

This error log serves as the primary retrieval corpus for the project. Every entry must strictly follow this 4-part structure:

- **Error:** The exact compiler output or runtime failure (e.g., `expected ObjectId, found &WlSurface`).
- **File:** The files where the error occurred or the fix was applied (e.g., `src-tauri/src/platform/linux/layer_shell/surface.rs`).
- **Root Cause:** A technically detailed explanation of *why* the error happened. Do not guess; trace the pointer or lifetime bounds.
- **Fix:** The exact code changes or architectural shifts implemented to resolve it.

## 3. Architecture Rules

- **No Framework Wrappers:** We avoid high-level abstractions that hide the Wayland event loop. If a crate hides the raw `wl_display` or `wl_surface` pointers needed by `khronos-egl`, we should usually fork it to modify it to expose the pointers or write it ourselves?.
- **Event Loop Multiplexing:** MPV render callbacks and Wayland shell events must be multiplexed into the same `calloop` event loop to avoid multithreading collisions on the single EGL OpenGL context. Do not spawn a detached thread for rendering unless strictly isolated.

## 4. Submitting Pull Requests

Ensure all GitHub Actions (`rust-clippy`, `semgrep`, `osv-scanner`) pass cleanly. 
We compile with `#![deny(warnings)]` enabled in CI, so any unused variables or unhandled `Result` variants will fail the build.
