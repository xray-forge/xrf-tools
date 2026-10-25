//! Texture descriptors, said in words.
//!
//! One file per part of the description, each owning both the shape the viewer renders and the rule that produces it
//! from a `ThmFile` chunk — the same arrangement `crates/xrf-thm` gives the chunks themselves.

mod archive_thm_bump;
mod archive_thm_declared_size;
mod archive_thm_description;
mod archive_thm_detail;
mod archive_thm_file;
mod archive_thm_flag;
mod archive_thm_material;
mod archive_thm_parameters;
mod archive_thm_texture;
mod archive_thm_texture_type;

pub use archive_thm_description::ArchiveThmDescription;
