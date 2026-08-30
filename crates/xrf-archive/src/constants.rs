//! The `.db` volume layout, as both directions of this workspace read and write it.

/// High bit of a chunk id, set when the chunk's payload is compressed.
///
/// `CFS_CompressMark` in `xray-16/src/xrCore/FS.h`.
pub const CHUNK_ID_COMPRESSED_MASK: u32 = 1 << 31;
/// The chunk id itself, with the compression flag masked off.
pub const CHUNK_ID_MASK: u32 = !(1 << 31);

/// Width of a chunk's id field.
pub const CHUNK_ID_FIELD_SIZE: u64 = 4;

/// Width of a chunk's size field, which a writer leaves blank while its payload is still growing.
pub const CHUNK_SIZE_FIELD_SIZE: u64 = 4;

/// Bytes a chunk spends before any payload.
pub const CHUNK_HEADER_SIZE: u64 = CHUNK_ID_FIELD_SIZE + CHUNK_SIZE_FIELD_SIZE;

/// Chunk carrying file payloads back to back.
///
/// A reader resolves entries by descriptor offset and never looks this chunk up, but a volume without it holds
/// nothing, so the id belongs with the rest of the layout rather than with whoever writes it.
pub const CHUNK_ID_DATA: u32 = 0;

/// Chunk carrying the entry name table, of which a volume has one.
///
/// `CLocatorAPI::LoadArchive` reads this id, and it is the only one written.
pub const CHUNK_ID_FILE_DESCRIPTORS: u32 = 1;

/// Every chunk id accepted as an entry name table.
///
/// `0x86` appears in volumes this tooling has to open but in no reference engine tree, so its provenance is
/// unverified — accepted because dropping it would make those volumes unreadable, not because the format documents
/// it. Never written.
pub const CHUNK_ID_FILE_DESCRIPTORS_READ: [u32; 2] = [CHUNK_ID_FILE_DESCRIPTORS, 0x86];

/// Chunk carrying the `[header]` metadata that names the volume's entry point.
///
/// `CFS_HeaderChunkID` in `xray-16/src/xrCore/FS.h`, which `CLocatorAPI::ProcessArchive` opens to read
/// `[header] entry_point`. Also what marks an archive as not being ShoC: a headerless volume not named `xdb` is
/// decrypted as an encrypted Shadow of Chernobyl archive and turns into nonsense.
pub const CHUNK_ID_METADATA: u32 = 666;

/// Every chunk id accepted as that metadata. `1337` is unverified in the same way as `0x86` above, and never written.
pub const CHUNK_ID_METADATA_READ: [u32; 2] = [CHUNK_ID_METADATA, 1337];

/// The four numeric fields a descriptor row carries around its name.
///
/// This is what the row's leading size field counts: `archive_file_header::ELEMENTS_SIZE` in
/// `xray-16/src/xrCore/LocatorAPI.h`, whose comment reads "size of following members", so the field excludes itself.
pub const DESCRIPTOR_ROW_FIELDS_SIZE: u16 = 16;

/// Width of that leading field, which a row occupies but does not declare.
///
/// The two differ, and confusing them costs two bytes per row in either direction: a reader takes them off the name
/// and mangles it, a writer budgeting a volume under-reserves its table.
pub const DESCRIPTOR_ROW_SIZE_FIELD_SIZE: u16 = 2;

/// Upper bound on an entry name, matching the fixed buffer a volume's name table is read through.
///
/// A header declaring a longer name is rejected rather than truncated: a truncated name is a different asset, so it
/// would shadow or miss silently.
pub const MAXIMUM_ENTRY_NAME_SIZE: usize = 520;
