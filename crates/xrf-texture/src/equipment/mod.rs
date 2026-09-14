//! Inventory equipment sheets: the icons a section declares, and the three operations over them.

mod equipment_grid_rect;
mod equipment_icon_path;
mod equipment_ltx;
mod equipment_slot_claim;
mod equipment_slot_occupant;
mod inventory_sprite_descriptor;
mod pack_equipment_options;
mod pack_equipment_processor;
mod pack_equipment_result;
mod unpack_equipment_options;
mod unpack_equipment_processor;
mod verify_equipment_grid_processor;

#[cfg(test)]
mod tests;

pub use equipment_slot_claim::EquipmentSlotClaim;
pub use equipment_slot_occupant::EquipmentSlotOccupant;
pub use inventory_sprite_descriptor::InventorySpriteDescriptor;
pub use pack_equipment_options::PackEquipmentOptions;
pub use pack_equipment_processor::PackEquipmentProcessor;
pub use pack_equipment_result::PackEquipmentResult;
pub use unpack_equipment_options::UnpackEquipmentOptions;
pub use unpack_equipment_processor::UnpackEquipmentProcessor;
pub use verify_equipment_grid_processor::{EquipmentGridOverlap, VerifyEquipmentGridProcessor};

pub(crate) use equipment_grid_rect::EquipmentGridRect;
pub(crate) use equipment_icon_path::*;
pub(crate) use equipment_ltx::*;
