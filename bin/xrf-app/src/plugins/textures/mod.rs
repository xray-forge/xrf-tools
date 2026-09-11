//! Textures as the engine names them: the catalog of a root set, what one texture's descriptor makes of it, the bytes
//! a webview needs to show it, and the writing the editor does to all three.
//!
//! The modules below are a layered story rather than a flat set, and reading them in this order is the shortest way in.
//!
//! Addressing, which everything else is written on top of:
//!
//! - [`source`] - how a texture is named: an engine reference, or a file on disk.
//! - [`files`] - the two files one texture is, named from either half.
//!
//! Reading, which both applications do:
//!
//! - [`catalog`] - every texture a root set holds, listed by reference or by path.
//! - [`summary`] - what one descriptor makes of its texture, for the badges a listing carries.
//! - [`description`] - everything the panels say about one texture, resolved in a single call.
//! - [`vocabulary`] - the names the SDK gives the numbers a descriptor stores.
//!
//! Writing, which only the editor does:
//!
//! - [`descriptor_form`] - the descriptor fields a person edits, read off a `.thm` and written back onto one.
//! - [`edit_targets`] - where an edit of a texture would land.
//! - [`file_stamp`] - what a file looked like when it was read, so a write cannot overwrite a change it never saw.
//! - [`save`] - writing a node's two files, or refusing to.
//! - [`encoding`] - the candidate formats a texture could be written in, and the encodes a comparison keeps.
//!
//! And the plumbing every plugin has: [`request`] for the wire shapes commands take, [`lease`] for what a job holds
//! while it runs, [`state`] for what the plugin holds between calls, and [`plugin`] for assembly.

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
