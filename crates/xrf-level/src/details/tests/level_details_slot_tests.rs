use crate::details::level_details_light::LevelDetailsLight;
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

/// A corner's object is six bits of the slot's first two words, the light the twenty above them.
#[test]
fn reads_and_writes_a_slot_s_corners_and_light_as_the_editor_does() {
  let slot: [u32; 4] = [0x0410_03E8, 0xFFFF_F1FF, 0x1111, 0x2222];
  let corners = |slot: &[u32; 4]| -> Vec<Option<u8>> {
    (0..LevelDetailsSlot::CORNERS)
      .map(|corner| LevelDetailsSlot::get_object(slot, corner))
      .collect()
  };

  assert_eq!(corners(&slot), [Some(1), Some(1), None, Some(7)]);
  assert_eq!(
    corners(&LevelDetailsSlot::with_object(slot, 2, 5)),
    [Some(1), Some(1), Some(5), Some(7)]
  );
  assert_eq!(
    LevelDetailsSlot::get_light(&slot),
    LevelDetailsLight::from_values([15, 15, 15, 15, 15])
  );
  assert_eq!(
    LevelDetailsSlot::with_light(slot, LevelDetailsLight::UNLIT),
    [0x0410_03E8, 0x0000_01FF, 0x1111, 0x2222]
  );
}

#[test]
fn packs_each_light_value_into_its_own_four_bits() {
  let light: LevelDetailsLight = LevelDetailsLight::from_values([1, 2, 3, 4, 5]);
  let lit: [u32; 4] = LevelDetailsSlot::with_light(slot(1000, 25, [0, 1, 2, 3]), light);

  assert_eq!(lit[1] >> 12, 0x0005_4321);
  assert_eq!(LevelDetailsSlot::get_light(&lit), light);
  assert_eq!(
    LevelDetailsSlot::of(&lit),
    LevelDetailsSlot::of(&slot(1000, 25, [0, 1, 2, 3]))
  );
}

#[test]
fn clears_the_corners_whose_palette_plants_nothing() {
  // The first and fourth corners carry densities; the second and third are four zeros each.
  let words: [u32; 4] = slot(0, 0, [1, 2, 3, 4]);
  let words: [u32; 4] = [words[0], words[1], 0x0000_0001, 0x1000_0000];

  assert_eq!(
    LevelDetailsSlot::of(&LevelDetailsSlot::with_bare_corners_cleared(words)).objects,
    [Some(1), None, None, Some(4)]
  );
}

#[test]
fn quantizes_a_light_value_down_into_four_bits() {
  assert_eq!(LevelDetailsLight::quantize(0.0), 0);
  assert_eq!(LevelDetailsLight::quantize(0.066), 0);
  assert_eq!(LevelDetailsLight::quantize(0.5), 7);
  assert_eq!(LevelDetailsLight::quantize(1.0), 15);
  assert_eq!(LevelDetailsLight::quantize(3.0), 15);
  assert_eq!(LevelDetailsLight::quantize(-1.0), 0);
  assert_eq!(LevelDetailsLight::quantize(f32::NAN), 0);
}
