// core - app-level orchestration
//
// manages the lifecycle of sidecar processes, wallpaper state
// persistence, and any cross-cutting concerns that don't belong
// in a specific platform implementation.

pub mod state;
pub mod engine_video;
