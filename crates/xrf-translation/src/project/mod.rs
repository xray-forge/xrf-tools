//! Whole translation roots: how a tree of files is discovered, described, edited, and built.

pub(crate) mod build;
pub(crate) mod constants;
pub(crate) mod descriptor;
pub(crate) mod edit;
pub(crate) mod gamedata_read;
pub(crate) mod initialize;
pub(crate) mod layout;
pub(crate) mod parse;
pub(crate) mod source_read;
pub(crate) mod verify;

#[cfg(test)]
mod tests;
