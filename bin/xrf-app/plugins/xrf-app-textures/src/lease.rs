//! Action exclusion for texture encoding.

/// Build, bump generation and comparison saturate the shared pool; one encode run is admitted at a time.
pub const TEXTURE_ENCODE_GROUP: &str = "textures.encode";
