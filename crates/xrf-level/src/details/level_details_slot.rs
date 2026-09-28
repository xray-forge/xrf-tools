use serde::{Deserialize, Serialize};

/// One cell of a level's detail grid, `DetailSlot` (`Layers/xrRender/DetailFormat.h`), as far as a reader on the CPU
/// asks it: what it plants and how high. Its densities and light are decoded where they are used, on the GPU.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDetailsSlot {
  /// Detail objects planted in the slot's four corners, by index into the level's own library; `None` for a corner
  /// planting nothing.
  pub objects: [Option<u8>; 4],
  /// Packed ground height of the slot, in metres.
  pub base_height: f32,
  /// Packed height the planting occupies above the base, in metres.
  pub height: f32,
}

impl LevelDetailsSlot {
  /// Bytes one slot occupies: a packed word and a palette per corner.
  pub const SERIALIZED_SIZE: usize = 16;

  /// `u32` words one slot is read as: the bitfield's two, then the palettes two to a word.
  pub const WORDS: usize = 4;

  /// The id a slot uses to say a corner plants nothing, `DetailSlot::ID_Empty`.
  const EMPTY_ID: u8 = 0x3F;

  /// What one unit of the packed base height is worth, in metres.
  const BASE_STEP: f32 = 0.2;

  /// Where the packed base height counts from, in metres.
  const BASE_ORIGIN: f32 = -200.0;

  /// What one unit of the packed height is worth, in metres.
  const HEIGHT_STEP: f32 = 0.1;

  /// Reads one slot out of its words.
  pub fn of(words: &[u32; Self::WORDS]) -> Self {
    let word: u64 = u64::from(words[0]) | (u64::from(words[1]) << 32);

    let base: u32 = (word & 0xFFF) as u32;
    let height: u32 = ((word >> 12) & 0xFF) as u32;

    Self {
      objects: [20, 26, 32, 38].map(|shift| {
        let id: u8 = ((word >> shift) & 0x3F) as u8;

        (id != Self::EMPTY_ID).then_some(id)
      }),
      base_height: Self::BASE_ORIGIN + base as f32 * Self::BASE_STEP,
      height: height as f32 * Self::HEIGHT_STEP,
    }
  }

  /// Whether anything at all is planted here.
  pub fn is_planted(&self) -> bool {
    self.objects.iter().any(Option::is_some)
  }
}
