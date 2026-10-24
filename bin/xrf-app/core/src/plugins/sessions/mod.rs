//! Which tool in which window holds each opening, so the openings a tool, a page or a window leaves are closed in
//! every domain at once: the page's side of `crate::session`'s scopes.

pub(crate) mod commands;
mod plugin;

pub use plugin::SessionsPlugin;
