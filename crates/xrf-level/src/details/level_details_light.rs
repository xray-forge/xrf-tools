use serde::{Deserialize, Serialize};

/// A detail slot's light, `DetailSlot`'s `c_dir`, `c_hemi`, `c_r`, `c_g` and `c_b` (`xrDO_Light/DetailFormat.h`): four
/// bits each, from 0 to [`Self::MAXIMUM`], the sun's, the sky's and the static lights' colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDetailsLight {
  pub sun: u8,
  pub hemi: u8,
  pub red: u8,
  pub green: u8,
  pub blue: u8,
}

impl LevelDetailsLight {
  /// The largest value four bits hold.
  pub const MAXIMUM: u8 = 15;

  /// No light at all, as the Build exports every slot for the detail lighter to light.
  pub const UNLIT: Self = Self {
    sun: 0,
    hemi: 0,
    red: 0,
    green: 0,
    blue: 0,
  };

  /// A light value from 0 to 1 in four bits, `DetailSlot::w_qclr(value, 15)`: the product rounded down, clamped, and
  /// 0 for no number at all.
  pub fn quantize(value: f32) -> u8 {
    ((value * f32::from(Self::MAXIMUM)).floor() as i32).clamp(0, i32::from(Self::MAXIMUM)) as u8
  }

  /// The five values in the order the slot packs them, from bit 44 up.
  pub const fn get_values(self) -> [u8; 5] {
    [self.sun, self.hemi, self.red, self.green, self.blue]
  }

  /// The light of five values in the order the slot packs them, each kept to its four bits.
  pub const fn from_values(values: [u8; 5]) -> Self {
    Self {
      sun: values[0] & Self::MAXIMUM,
      hemi: values[1] & Self::MAXIMUM,
      red: values[2] & Self::MAXIMUM,
      green: values[3] & Self::MAXIMUM,
      blue: values[4] & Self::MAXIMUM,
    }
  }
}
