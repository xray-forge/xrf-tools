//! The chunk header, as every chunked X-Ray format lays it out.
//!
//! Eight bytes of id and size open every chunk of every format the engine reads, a `.db` volume included, so the
//! numbers live with the reader that cuts them rather than with any one format built on top.

/// Width of a chunk's id field.
pub const CHUNK_ID_FIELD_SIZE: u64 = 4;

/// Width of a chunk's size field, which a writer leaves blank while its payload is still growing.
pub const CHUNK_SIZE_FIELD_SIZE: u64 = 4;

/// Bytes a chunk spends before any payload.
pub const CHUNK_HEADER_SIZE: u64 = CHUNK_ID_FIELD_SIZE + CHUNK_SIZE_FIELD_SIZE;

/// High bit of a chunk id, set when the chunk's payload is compressed.
///
/// `CFS_CompressMark` in `xray-16/src/xrCore/FS.h`.
pub const CHUNK_ID_COMPRESSED_MASK: u32 = 1 << 31;

/// The chunk id itself, with the compression flag masked off.
///
/// The engine compares ids this way rather than exactly (`xrCore/FS.h`), so a reader that demanded the marked
/// spelling would miss a chunk the engine finds.
pub const CHUNK_ID_MASK: u32 = !CHUNK_ID_COMPRESSED_MASK;
