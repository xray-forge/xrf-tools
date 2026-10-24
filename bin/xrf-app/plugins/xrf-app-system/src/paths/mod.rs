//! Filesystem paths as this desktop application addresses them: what one currently holds, where it can be shown, and
//! where tools write when nobody named a destination.

pub mod commands;
pub mod path_description;
pub mod reveal;

pub use path_description::PathDescription;
