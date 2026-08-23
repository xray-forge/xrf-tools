//! The Tauri plugins the application exposes, one module per command domain.
//!
//! A domain owns its commands, the plugin that dispatches them, and whatever state they share. Wire names and
//! command paths are declared once in `registry`, which derives runtime, Specta, and ACL registration from the
//! same tokens.

pub mod archives;
pub mod assets;
pub mod configs;
pub mod dialogs;
pub mod equipment_icons;
pub mod exports;
pub mod spawn;
pub mod system;
pub mod translations;
pub mod visuals;
