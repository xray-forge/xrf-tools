use std::sync::{Arc, Mutex};

use serde::Serialize;
use xrf_texture::InventorySpriteDescriptor;

pub struct EquipmentSpriteState {
  pub system_ltx_path: Arc<Mutex<Option<String>>>,
  pub equipment_sprite_path: Arc<Mutex<Option<String>>>,
  pub equipment_sprite_name: Arc<Mutex<Option<String>>>,
  pub equipment_sprite_preview: Arc<Mutex<Option<Vec<u8>>>>,
  pub equipment_descriptors: Arc<Mutex<Option<Vec<InventorySpriteDescriptor>>>>,
}

impl EquipmentSpriteState {
  pub fn new() -> Self {
    Self {
      system_ltx_path: Arc::new(Mutex::new(None)),
      equipment_sprite_path: Arc::new(Mutex::new(None)),
      equipment_sprite_name: Arc::new(Mutex::new(None)),
      equipment_sprite_preview: Arc::new(Mutex::new(None)),
      equipment_descriptors: Arc::new(Mutex::new(None)),
    }
  }
}

#[derive(Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSpriteMetadata {
  pub path: String,
  pub name: String,
  pub system_ltx_path: String,
  pub equipment_descriptors: Vec<InventorySpriteDescriptor>,
}
