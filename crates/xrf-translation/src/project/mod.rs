//! Whole translation roots: how a tree of files is described, edited, built, verified, imported and formatted.

pub(crate) mod build;
pub(crate) mod describe;
pub(crate) mod format;
pub(crate) mod initialize;
pub(crate) mod parse;
pub(crate) mod translation_editor;
pub(crate) mod verify;

#[cfg(test)]
pub(crate) mod tests;
