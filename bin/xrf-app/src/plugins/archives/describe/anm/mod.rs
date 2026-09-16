//! Object motions, said in words.
//!
//! An animation is a header and six envelopes, and everything a reader wants of it is a total taken over keys: how
//! long it runs, how far the keys reach, and what each channel does at either end of them.

mod archive_anm_behavior;
mod archive_anm_channel;
mod archive_anm_description;
mod archive_anm_shape;

pub use archive_anm_description::ArchiveAnmDescription;
