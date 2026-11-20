//! What a sprite sheet run calls the work it is doing.
//!
//! Beside each other rather than in four option modules, because they are one vocabulary: whichever surface is
//! watching sees the same words across the four commands.

/// Phase an equipment sprite pack reports while it draws sections into the sheet.
pub const SPRITE_PHASE_PACK_EQUIPMENT: &str = "pack";

/// Phase an equipment sprite unpack reports while it cuts sections out of the sheet.
pub const SPRITE_PHASE_UNPACK_EQUIPMENT: &str = "unpack";

/// Phase a description pack reports while it writes each described texture.
pub const SPRITE_PHASE_PACK_DESCRIPTIONS: &str = "pack";

/// Phase a description unpack reports while it reads each described texture.
pub const SPRITE_PHASE_UNPACK_DESCRIPTIONS: &str = "unpack";
