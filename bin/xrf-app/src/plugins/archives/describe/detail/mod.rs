//! Detail objects, said in words.
//!
//! Grouped by format rather than by file name: `level.details` is a level file and a `.dm` is not, but the two hold
//! the same record and share the description of it.

mod archive_detail_library_description;
mod archive_detail_model;

pub use archive_detail_library_description::ArchiveDetailLibraryDescription;
pub use archive_detail_model::ArchiveDetailModel;
