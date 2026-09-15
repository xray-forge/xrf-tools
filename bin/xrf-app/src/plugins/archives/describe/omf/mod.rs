//! Motion banks, said in words.
//!
//! A bank is a collection where a descriptor is a record: one partition and up to eleven hundred motions, and what a
//! reader wants of it is the list. Each file here owns one part of that list and the rule producing it.

mod archive_omf_bank;
mod archive_omf_blend;
mod archive_omf_description;
mod archive_omf_mark;
mod archive_omf_motion;
mod archive_omf_motion_flag;
mod archive_omf_part;
mod archive_omf_quantized;
mod archive_omf_target;

pub use archive_omf_description::ArchiveOmfDescription;
