//! What a texture run calls the work it is doing.
//!
//! Beside each other rather than in each option module, because they are one vocabulary: whichever surface is watching
//! sees the same words across the commands.

/// Phase a bump generation reports as it walks the steps `DXTCompressBump` walks.
pub const TEXTURE_PHASE_GENERATE_BUMP: &str = "generate";

/// Phase a texture build reports while it encodes the one image it was given.
pub const TEXTURE_PHASE_BUILD_TEXTURE: &str = "build";
