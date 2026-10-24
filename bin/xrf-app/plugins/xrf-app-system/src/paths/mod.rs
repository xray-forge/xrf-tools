//! Filesystem paths as this desktop application addresses them: what one currently holds, where it can be shown, and
//! where tools write when nobody named a destination.

pub(crate) mod commands;
mod path_description;
mod path_reveal;
#[cfg(test)]
mod tests;

pub use path_description::{PathDescription, PathKind};
pub use path_reveal::PathReveal;
