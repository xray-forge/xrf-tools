//! The Tauri plugins the application exposes, one module per command domain.
//!
//! A domain owns its commands, the plugin that dispatches them, and whatever state they share. Wire names and
//! command paths are declared once in `ipc::registry`, which derives runtime, Specta, and ACL registration from the
//! same tokens. Which of these domains the application actually assembles is `registry`.

pub mod archives;
pub mod assets;
pub mod configs;
pub mod dialogs;
pub mod exports;
pub mod gamedata;
pub mod jobs;
pub mod registry;
pub mod spawn;
pub mod sprite_equipment;
pub mod system;
pub mod textures;
pub mod translations;
pub mod visuals;
