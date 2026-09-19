//! The `.db` volume layout, as both directions of this workspace read and write it.

/// Chunk carrying file payloads back to back.
pub const CHUNK_ID_DATA: u32 = 0;
/// Chunk carrying the entry name table, of which a volume has one.
pub const CHUNK_ID_FILE_DESCRIPTORS: u32 = 1;
/// Every chunk id accepted as an entry name table.
pub const CHUNK_ID_FILE_DESCRIPTORS_READ: [u32; 2] = [CHUNK_ID_FILE_DESCRIPTORS, 0x86];
/// Chunk carrying the `[header]` metadata that names the volume's entry point.
pub const CHUNK_ID_METADATA: u32 = 666;
/// Every chunk id accepted as that metadata. `1337` is unverified in the same way as `0x86` above, and never written.
pub const CHUNK_ID_METADATA_READ: [u32; 2] = [CHUNK_ID_METADATA, 1337];

/// The four numeric fields a descriptor row carries around its name.
pub const DESCRIPTOR_ROW_FIELDS_SIZE: u16 = 16;
/// Width of that leading field, which a row occupies but does not declare.
pub const DESCRIPTOR_ROW_SIZE_FIELD_SIZE: u16 = 2;

/// Upper bound on an entry name, matching the fixed buffer a volume's name table is read through.
pub const MAXIMUM_ENTRY_NAME_SIZE: usize = 520;
