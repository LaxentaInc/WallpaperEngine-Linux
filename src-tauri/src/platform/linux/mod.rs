// platform::linux - all linux-specific code lives under here
//
// each subfolder is a self-contained implementation for a specific
// compositor capability or shared utility. adding support for a new
// compositor means adding a new sibling folder, nothing else changes.

pub mod shared;
pub mod runner;
pub mod layer_shell;
pub mod x11;
pub mod mutter;
