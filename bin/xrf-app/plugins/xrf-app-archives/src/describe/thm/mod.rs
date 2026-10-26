//! Texture descriptors, said in words.
//!
//! One file per part of the description, each owning both the shape the viewer renders and the rule that produces it
//! from a `ThmFile` chunk — the same arrangement `crates/xrf-thm` gives the chunks themselves.

mod archive_thm_bump;
mod archive_thm_description;
mod archive_thm_detail;
mod archive_thm_material;
mod file;
mod parameters;
mod texture;

pub use archive_thm_description::ArchiveThmDescription;
