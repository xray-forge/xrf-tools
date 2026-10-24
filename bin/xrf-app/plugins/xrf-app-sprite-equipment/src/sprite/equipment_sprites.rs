use xrf_app_core::session::Sessions;

use crate::EquipmentSpriteDocument;

/// The sheets open, each held by the tool that opened it; a reload is a handle of its own, which replaces the one it
/// reloaded once it commits.
pub type EquipmentSprites = Sessions<EquipmentSpriteDocument>;
