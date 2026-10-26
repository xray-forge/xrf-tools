//! What an opened translations root holds, and the two layouts it is read in.

pub(crate) mod translation_file;
pub(crate) mod translation_finding;
pub(crate) mod translation_gamedata_reader;
pub(crate) mod translation_project_descriptor;
pub(crate) mod translation_project_mode;
pub(crate) mod translation_source;
pub(crate) mod translation_source_reader;

#[cfg(test)]
mod tests;
