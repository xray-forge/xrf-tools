//! Textures as the engine names them: the catalog of a root set, what one texture's descriptor makes of it, the bytes
//! a webview needs to show it, and the writing the editor does to all three.

pub mod catalog;
pub mod commands;
pub mod description;
pub mod descriptor_form;
pub mod edit_targets;
pub mod encoding;
pub mod file_stamp;
pub mod files;
pub mod lease;
pub mod plugin;
pub mod request;
pub mod save;
pub mod source;
pub mod state;
pub mod summary;
pub mod vocabulary;

#[cfg(test)]
mod tests;
