//! Saying in words what an entry the viewer cannot draw contains.
//!
//! One describer per format, each answering with a shape built for reading rather than the shape the format has on
//! disk. Everything a describer needs of the explorer arrives through [`ArchiveDescribeSource`], so a describer is
//! written against its format and not against the two subjects the explorer browses.

mod animation;
mod anm;
mod archive_describe_scope;
mod archive_describe_source;
mod archive_described_format;
mod archive_entry_reader;
mod archive_file_description;
mod archive_reference;
mod chunks;
pub mod commands;
mod level;
mod omf;
mod particles;
mod ppe;
mod shaders;
mod spawn;
mod thm;

pub use archive_describe_source::ArchiveDescribeSource;
pub use archive_file_description::ArchiveFileDescription;
