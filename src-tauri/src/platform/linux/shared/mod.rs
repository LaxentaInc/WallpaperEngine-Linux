// platform::linux::shared - types, traits, and utilities shared across all shells
//
// every compositor implementation (layer_shell, x11, mutter) imports from here.
// this is the contract layer: it defines WHAT a desktop surface must do,
// but never HOW (that's each shell folder's job).

pub mod surface_trait;
pub mod types;
pub mod detection;
pub mod ipc;
