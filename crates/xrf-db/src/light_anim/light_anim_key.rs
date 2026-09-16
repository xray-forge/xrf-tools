use serde::{Deserialize, Serialize};

/// One keyed colour of a light animation, as `CLAItem::Keys` holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimKey {
  /// The frame the colour is held from, which the engine divides by the item's own rate to reach a time.
  pub frame: u32,
  /// The colour as stored. A version 0 file holds it as BGR, which the engine swaps on load.
  pub color: u32,
}

impl LightAnimKey {
  /// Bytes one key occupies.
  pub const SERIALIZED_SIZE: u64 = 4 + 4;

  /// The red channel, taking the colour as the engine leaves it after load.
  pub const fn get_red(&self) -> u8 {
    (self.color >> 16) as u8
  }

  pub const fn get_green(&self) -> u8 {
    (self.color >> 8) as u8
  }

  pub const fn get_blue(&self) -> u8 {
    self.color as u8
  }

  pub const fn get_alpha(&self) -> u8 {
    (self.color >> 24) as u8
  }
}
