use crate::details::level_details_slot::LevelDetailsSlot;

/// A slot's words, its bitfield packed in the order the engine packs it and its palettes left clear.
fn slot(base: u64, height: u64, objects: [u64; 4]) -> [u32; LevelDetailsSlot::WORDS] {
  let word: u64 =
    base | (height << 12) | (objects[0] << 20) | (objects[1] << 26) | (objects[2] << 32) | (objects[3] << 38);

  [word as u32, (word >> 32) as u32, 0, 0]
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
