use xrf_app_core::session::Sessions;

use crate::document::EquipmentSpriteDocument;

/// The sheets open, each held by the tool that opened it, each published and replaced as a whole.
pub type EquipmentSpriteState = Sessions<EquipmentSpriteDocument>;
