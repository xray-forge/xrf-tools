//! Inventory equipment sheets: the icons a section declares, and the three operations over them.

mod equipment_grid_overlap;
mod equipment_grid_rect;
mod equipment_grid_verifier;
mod equipment_icon_path;
mod equipment_ltx;
mod equipment_slot_claim;
mod equipment_slot_occupant;
mod equipment_sprite_pack_options;
mod equipment_sprite_pack_result;
mod equipment_sprite_packer;
mod equipment_sprite_unpack_options;
mod equipment_sprite_unpacker;
mod inventory_sprite_descriptor;

#[cfg(test)]
mod tests;

pub use equipment_grid_overlap::EquipmentGridOverlap;
pub(crate) use equipment_grid_rect::EquipmentGridRect;
pub use equipment_grid_verifier::EquipmentGridVerifier;
pub(crate) use equipment_icon_path::*;
pub(crate) use equipment_ltx::*;
pub use equipment_slot_claim::EquipmentSlotClaim;
pub use equipment_slot_occupant::EquipmentSlotOccupant;
pub use equipment_sprite_pack_options::EquipmentSpritePackOptions;
pub use equipment_sprite_pack_result::EquipmentSpritePackResult;
pub use equipment_sprite_packer::EquipmentSpritePacker;
pub use equipment_sprite_unpack_options::EquipmentSpriteUnpackOptions;
pub use equipment_sprite_unpacker::EquipmentSpriteUnpacker;
pub use inventory_sprite_descriptor::InventorySpriteDescriptor;
