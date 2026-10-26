//! The two files a packing configuration is authored in, each read and written by one module.
//!
//! The xrCompress LTX dialect and JSON carry the same file-owned fields; which one a path holds is
//! [`crate::ArchiveConfigFormat`]'s to say.

mod archive_pack_config_codec;
mod archive_pack_config_json;
mod archive_pack_config_ltx;

pub use archive_pack_config_json::ArchivePackConfigJson;
