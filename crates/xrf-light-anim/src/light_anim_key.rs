use serde::{Deserialize, Serialize};

/// One keyed colour of a light animation, as `CLAItem::Keys` holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimKey {
  /// The frame the colour stands at, an `int` in the engine: shipped libraries hold keys before frame 0, such as
  /// `svet_01`'s at `-1`. The engine divides it by the item's own rate to reach a time.
  pub frame: i32,
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

  /// The same key with red and blue exchanged and alpha kept, as `ELightAnimLibrary::Load` turns a version 0 colour
  /// into the one it uses (`subst_alpha(bgr2rgb(c), color_get_A(c))`).
  pub const fn to_swapped_red_blue(self) -> Self {
    let red_blue: u32 = ((self.color & 0x00FF_0000) >> 16) | ((self.color & 0x0000_00FF) << 16);

    Self {
      frame: self.frame,
      color: (self.color & 0xFF00_FF00) | red_blue,
    }
  }
}
