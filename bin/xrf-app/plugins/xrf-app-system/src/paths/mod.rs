//! Filesystem paths as this desktop application addresses them: what one currently holds, where it can be shown,
//! where tools write when nobody named a destination, and what the application keeps of its own.

pub(crate) mod commands;
mod description;
mod reveal;
mod storage;
#[cfg(test)]
mod tests;

pub use description::PathDescription;
pub use reveal::PathReveal;
pub use storage::{AppStorageKind, AppStorageUsage};
