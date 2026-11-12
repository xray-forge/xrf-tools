use serde::{Deserialize, Serialize};

use crate::details::level_details_light::LevelDetailsLight;

/// One cell of a level's detail grid, `DetailSlot` (`Layers/xrRender/DetailFormat.h`), as far as a reader on the CPU
/// asks it: what it plants and how high. Its densities and light are decoded where they are used, on the GPU.
///
/// The associated functions read and write a slot's packed words, the one rule every crate packs a slot by: the first
/// two words hold its base height (12 bits), height (8), four corners' objects (6 each) and light (4 each, from bit
/// 44); the last two its corners' palettes, 16 bits each, four densities of 4 bits.
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

  /// Corners a slot plants, an object each.
  pub const CORNERS: usize = 4;

  /// The id a slot uses to say a corner plants nothing, `DetailSlot::ID_Empty`.
  pub const EMPTY_ID: u8 = 0x3F;

  /// The bit a slot's first corner's object starts at, each next corner six bits on.
  const OBJECT_SHIFT: usize = 20;

  /// The bit a slot's light starts at, `c_dir`, each next value four bits on.
  const LIGHT_SHIFT: usize = 44;

  /// What one unit of the packed base height is worth, in metres.
  const BASE_STEP: f32 = 0.2;

  /// Where the packed base height counts from, in metres.
  const BASE_ORIGIN: f32 = -200.0;

  /// What one unit of the packed height is worth, in metres.
  const HEIGHT_STEP: f32 = 0.1;

  /// Reads one slot out of its words.
  pub fn of(words: &[u32; Self::WORDS]) -> Self {
    let word: u64 = Self::get_bitfield(words);

    Self {
      objects: std::array::from_fn(|corner| Self::get_object(words, corner)),
      base_height: Self::BASE_ORIGIN + (word & 0xFFF) as f32 * Self::BASE_STEP,
      height: ((word >> 12) & 0xFF) as f32 * Self::HEIGHT_STEP,
    }
  }

  /// Whether anything at all is planted here.
  pub fn is_planted(&self) -> bool {
    self.objects.iter().any(Option::is_some)
  }

  /// The object a slot's corner plants, by its index in the library; `None` for a corner planting nothing.
  ///
  /// # Panics
  ///
  /// Panics in debug builds for a corner past [`Self::CORNERS`].
  pub fn get_object(words: &[u32; Self::WORDS], corner: usize) -> Option<u8> {
    debug_assert!(corner < Self::CORNERS, "a slot has four corners, not {corner}");

    let id: u8 = ((Self::get_bitfield(words) >> (Self::OBJECT_SHIFT + 6 * corner)) & 0x3F) as u8;

    (id != Self::EMPTY_ID).then_some(id)
  }

  /// `words` planting `object` in its corner `corner`, [`Self::EMPTY_ID`] for none, as `DetailSlot::w_id` writes it.
  ///
  /// # Panics
  ///
  /// Panics in debug builds for a corner past [`Self::CORNERS`].
  pub fn with_object(words: [u32; Self::WORDS], corner: usize, object: u8) -> [u32; Self::WORDS] {
    debug_assert!(corner < Self::CORNERS, "a slot has four corners, not {corner}");

    let shift: usize = Self::OBJECT_SHIFT + 6 * corner;
    let word: u64 = (Self::get_bitfield(&words) & !(0x3F << shift)) | (u64::from(object & 0x3F) << shift);

    Self::with_bitfield(words, word)
  }

  /// Whether a corner's palette plants anything: any of its four densities above zero.
  ///
  /// # Panics
  ///
  /// Panics in debug builds for a corner past [`Self::CORNERS`].
  pub fn has_density(words: &[u32; Self::WORDS], corner: usize) -> bool {
    debug_assert!(corner < Self::CORNERS, "a slot has four corners, not {corner}");

    (words[2 + corner / 2] >> (16 * (corner % 2))) & 0xFFFF != 0
  }

  /// `words` with each corner whose densities are all zero planting nothing, as the detail lighter leaves every slot
  /// (`process_pallete`, `xrDO_Light/DetailFormat.h:232`).
  pub fn with_bare_corners_cleared(words: [u32; Self::WORDS]) -> [u32; Self::WORDS] {
    (0..Self::CORNERS).fold(words, |words, corner| {
      if Self::has_density(&words, corner) {
        words
      } else {
        Self::with_object(words, corner, Self::EMPTY_ID)
      }
    })
  }

  /// A slot's light.
  pub fn get_light(words: &[u32; Self::WORDS]) -> LevelDetailsLight {
    let word: u64 = Self::get_bitfield(words);

    LevelDetailsLight::from_values(std::array::from_fn(|value| {
      ((word >> (Self::LIGHT_SHIFT + 4 * value)) & 0xF) as u8
    }))
  }

  /// `words` lit by `light`, [`LevelDetailsLight::UNLIT`] for the zeroed light the Build exports.
  pub fn with_light(words: [u32; Self::WORDS], light: LevelDetailsLight) -> [u32; Self::WORDS] {
    let word: u64 = light.get_values().iter().enumerate().fold(
      Self::get_bitfield(&words) & ((1 << Self::LIGHT_SHIFT) - 1),
      |word, (index, value)| word | (u64::from(*value) << (Self::LIGHT_SHIFT + 4 * index)),
    );

    Self::with_bitfield(words, word)
  }

  /// The slot's first two words as the one 64-bit field they pack.
  fn get_bitfield(words: &[u32; Self::WORDS]) -> u64 {
    u64::from(words[0]) | (u64::from(words[1]) << 32)
  }

  fn with_bitfield(words: [u32; Self::WORDS], word: u64) -> [u32; Self::WORDS] {
    [word as u32, (word >> 32) as u32, words[2], words[3]]
  }
}
