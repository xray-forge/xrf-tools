//! Filesystem paths as this desktop application addresses them: what one currently holds, where it can be shown,
//! where tools write when nobody named a destination, and what the application keeps of its own.

pub(crate) mod commands;
mod description;
mod reveal;
mod storage;
#[cfg(test)]
mod tests;

pub(crate) use description::PathDescription;
pub(crate) use reveal::PathReveal;
pub(crate) use storage::{AppStorageKind, AppStorageUsage};
