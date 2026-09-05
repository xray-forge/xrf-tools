//! Textures as the engine names them: the catalog of a root set, what one texture's descriptor makes of it, and the
//! bytes a webview needs to show it.

pub mod catalog;
pub mod commands;
pub mod description;
pub mod plugin;
pub mod source;
pub mod state;
pub mod summary;

#[cfg(test)]
mod tests;
