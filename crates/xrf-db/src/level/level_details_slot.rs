use serde::{Deserialize, Serialize};

/// The id a slot uses to say a corner plants nothing, `DetailSlot::ID_Empty`.
const EMPTY_ID: u8 = 0x3F;

/// What one unit of the packed base height is worth, in metres.
const BASE_STEP: f32 = 0.2;

/// Where the packed base height counts from, in metres.
const BASE_ORIGIN: f32 = -200.0;

/// What one unit of the packed height is worth, in metres.
const HEIGHT_STEP: f32 = 0.1;

/// One cell of a level's detail grid, `DetailSlot` (`Layers/xrRender/DetailFormat.h`).
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

  /// Reads one slot out of the grid's stored bytes.
  pub fn of(bytes: &[u8; Self::SERIALIZED_SIZE]) -> Self {
    let word: u64 = u64::from_le_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);

    let base: u32 = (word & 0xFFF) as u32;
    let height: u32 = ((word >> 12) & 0xFF) as u32;

    Self {
      objects: [20, 26, 32, 38].map(|shift| {
        let id: u8 = ((word >> shift) & 0x3F) as u8;

        (id != EMPTY_ID).then_some(id)
      }),
      base_height: BASE_ORIGIN + base as f32 * BASE_STEP,
      height: height as f32 * HEIGHT_STEP,
    }
  }

  /// Whether anything at all is planted here.
  pub fn is_planted(&self) -> bool {
    self.objects.iter().any(Option::is_some)
  }
}

#[cfg(test)]
mod tests {
  use super::LevelDetailsSlot;

  /// A slot's stored word, in the order the bitfield packs it.
  fn slot(base: u64, height: u64, objects: [u64; 4]) -> [u8; LevelDetailsSlot::SERIALIZED_SIZE] {
    let word: u64 =
      base | (height << 12) | (objects[0] << 20) | (objects[1] << 26) | (objects[2] << 32) | (objects[3] << 38);

    let mut bytes: [u8; LevelDetailsSlot::SERIALIZED_SIZE] = [0; LevelDetailsSlot::SERIALIZED_SIZE];

    bytes[..8].copy_from_slice(&word.to_le_bytes());

    bytes
  }

  #[test]
  fn an_empty_slot_plants_nothing_rather_than_object_sixty_three() {
    let described: LevelDetailsSlot = LevelDetailsSlot::of(&slot(0, 0, [0x3F; 4]));

    assert_eq!(described.objects, [None; 4]);
    assert!(!described.is_planted());
  }

  #[test]
  fn a_slot_names_the_object_in_each_corner_it_plants() {
    let described: LevelDetailsSlot = LevelDetailsSlot::of(&slot(0, 0, [0, 0x3F, 7, 0x3F]));

    assert_eq!(described.objects, [Some(0), None, Some(7), None]);
    assert!(described.is_planted());
  }

  #[test]
  fn a_height_reads_as_the_metres_the_engine_packs_it_from() {
    // 1000 base units from -200 m, and 25 height units, which is what the packer's own steps are worth.
    let described: LevelDetailsSlot = LevelDetailsSlot::of(&slot(1000, 25, [0x3F; 4]));

    assert_eq!(described.base_height, 0.0);
    assert_eq!(described.height, 2.5);
  }
}
