//! Saying in words what an entry the viewer cannot draw contains.
//!
//! One describer per format, each answering with a shape built for reading rather than the shape the format has on
//! disk. Everything a describer needs of the explorer arrives through [`ArchiveDescribeSource`], so a describer is
//! written against its format and not against the two subjects the explorer browses.

mod animation;
mod anm;
mod chunks;
pub(crate) mod commands;
mod description;
mod detail;
mod efd;
mod gamemtl;
mod level;
mod light_anim;
mod omf;
mod particles;
mod ppe;
mod shader_compiler;
mod shaders;
mod sound;
mod source;
mod spawn;
#[cfg(test)]
mod tests;
mod thm;

pub use description::ArchiveFileDescription;
pub use source::ArchiveDescribeSource;
