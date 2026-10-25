//! Whole translation roots: how a tree of files is discovered, described, edited, and built.

pub(crate) mod build;
pub(crate) mod constants;
pub(crate) mod edit;
pub(crate) mod format;
pub(crate) mod gamedata_read;
pub(crate) mod initialize;
pub(crate) mod job_phases;
pub(crate) mod layout;
pub(crate) mod parse;
pub(crate) mod source_read;
pub(crate) mod translation_file;
pub(crate) mod translation_finding;
pub(crate) mod translation_project_descriptor;
pub(crate) mod translation_project_mode;
pub(crate) mod translation_source;
pub(crate) mod verify;

#[cfg(test)]
mod tests;
