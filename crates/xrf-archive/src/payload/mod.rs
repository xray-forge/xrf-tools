//! Access to stored entry payloads, including decompression, copying, and shared payload identification.

mod archive_located_entry;
mod archive_open_volume;
mod archive_open_volumes;
mod archive_shared_payload;

pub use archive_open_volumes::ArchiveOpenVolumes;
pub use archive_shared_payload::ArchiveSharedPayload;
