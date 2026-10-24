use crate::core::session::Sessions;
use crate::plugins::sprite_equipment::document::EquipmentSpriteDocument;

/// The sheets open, each held by the tool that opened it, each published and replaced as a whole.
pub type EquipmentSpriteState = Sessions<EquipmentSpriteDocument>;
