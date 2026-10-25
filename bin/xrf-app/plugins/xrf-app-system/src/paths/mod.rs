//! Filesystem paths as this desktop application addresses them: what one currently holds, where it can be shown, and
//! where tools write when nobody named a destination.

pub(crate) mod commands;
mod description;
mod reveal;
#[cfg(test)]
mod tests;

pub use description::PathDescription;
pub use reveal::PathReveal;
