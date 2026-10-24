//! Action exclusion and progress for texture encoding.

/// Build, bump generation and comparison saturate the shared pool; one encode run is admitted at a time.
pub const TEXTURE_ENCODE_GROUP: &str = "textures.encode";

/// Comparison is the app's loop over candidates; the image crate encodes one candidate at a time.
pub const TEXTURE_PHASE_WEIGH: &str = "weigh";
