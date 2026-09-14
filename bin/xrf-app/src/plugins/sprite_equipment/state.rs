use crate::core::session::Session;
use crate::plugins::sprite_equipment::document::EquipmentSpriteDocument;

/// The one sheet this plugin holds open, published and replaced as a whole.
pub type EquipmentSpriteState = Session<EquipmentSpriteDocument>;
