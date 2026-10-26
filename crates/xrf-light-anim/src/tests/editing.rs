use xrf_error::{XrfError, XrfResult};

use crate::light_anim_file::LightAnimFile;
use crate::light_anim_item::LightAnimItem;
use crate::light_anim_key::LightAnimKey;

fn frames(item: &LightAnimItem) -> Vec<i32> {
  item.keys.iter().map(|key| key.frame).collect()
}

fn key(frame: i32, color: u32) -> LightAnimKey {
  LightAnimKey { frame, color }
}

#[test]
fn starts_a_new_animation_as_the_editor_does() {
  let item: LightAnimItem = LightAnimItem::new("koster");

  assert_eq!(item.fps, LightAnimItem::DEFAULT_FPS);
  assert_eq!(item.frame_count, LightAnimItem::DEFAULT_FRAME_COUNT);
  assert_eq!(item.keys, [key(0, 0)]);
}

#[test]
fn inserts_a_key_in_frame_order_and_replaces_one_already_there() -> XrfResult {
  let mut item: LightAnimItem = LightAnimItem::new("koster");

  item.resize(10)?;
  item.insert_key(6, 0x0000_0006)?;
  item.insert_key(3, 0x0000_0003)?;
  item.insert_key(6, 0x0000_0060)?;

  assert_eq!(frames(&item), [0, 3, 6]);
  assert_eq!(item.keys[2].color, 0x0000_0060);

  Ok(())
}

#[test]
fn refuses_a_frame_past_the_frame_count() {
  let mut item: LightAnimItem = LightAnimItem::new("koster");

  assert!(matches!(item.insert_key(2, 0), Err(XrfError::Invalid { .. })));
  assert!(item.delete_key(2).is_err());
  assert!(item.move_key(0, 2).is_err());
}

#[test]
fn deletes_any_key_but_the_one_at_frame_zero() -> XrfResult {
  let mut item: LightAnimItem = LightAnimItem::new("koster");

  item.resize(10)?;
  item.insert_key(4, 0x0000_0004)?;

  assert!(item.delete_key(4)?);
  assert!(!item.delete_key(4)?, "nothing is left at frame 4");
  assert!(!item.delete_key(0)?, "frame 0 always keeps its key");
  assert_eq!(frames(&item), [0]);

  Ok(())
}

#[test]
fn moves_a_key_over_another_and_onto_its_own_frame_without_loss() -> XrfResult {
  let mut item: LightAnimItem = LightAnimItem::new("koster");

  item.resize(10)?;
  item.insert_key(2, 0x0000_0002)?;
  item.insert_key(5, 0x0000_0005)?;

  assert!(item.move_key(2, 5)?);
  assert_eq!(item.keys, [key(0, 0), key(5, 0x0000_0002)]);

  assert!(item.move_key(5, 5)?);
  assert_eq!(item.keys, [key(0, 0), key(5, 0x0000_0002)]);

  assert!(!item.move_key(7, 8)?, "nothing is keyed at frame 7");

  Ok(())
}

#[test]
fn grows_by_moving_the_key_at_the_old_end_to_the_new_one() -> XrfResult {
  let mut item: LightAnimItem = LightAnimItem::new("koster");

  item.insert_key(1, 0x0000_0001)?;
  item.resize(8)?;

  assert_eq!(item.frame_count, 8);
  assert_eq!(item.keys, [key(0, 0), key(8, 0x0000_0001)]);

  Ok(())
}

#[test]
fn shrinks_by_dropping_every_key_from_the_new_frame_count_on() -> XrfResult {
  let mut item: LightAnimItem = LightAnimItem::new("koster");

  item.resize(10)?;
  item.insert_key(-1, 0x0000_00FF)?;
  item.insert_key(4, 0x0000_0004)?;
  item.insert_key(5, 0x0000_0005)?;
  item.resize(5)?;

  assert_eq!(frames(&item), [-1, 0, 4]);
  assert!(item.resize(0).is_err(), "an animation has at least one frame");

  Ok(())
}

#[test]
fn steps_between_keys_as_the_editor_does() -> XrfResult {
  let mut item: LightAnimItem = LightAnimItem::new("koster");

  item.resize(20)?;
  item.insert_key(5, 0)?;
  item.insert_key(12, 0)?;

  assert_eq!(item.get_previous_key_frame(12), Some(5));
  assert_eq!(item.get_previous_key_frame(0), Some(0), "nothing before the first key");
  assert_eq!(item.get_previous_key_frame(30), Some(12), "past every key");
  assert_eq!(item.get_next_key_frame(5), Some(12));
  assert_eq!(item.get_next_key_frame(12), Some(12), "nothing after the last key");

  Ok(())
}

#[test]
fn appends_clones_renames_and_removes_animations_by_unique_name() -> XrfResult {
  let mut library: LightAnimFile = LightAnimFile {
    version: LightAnimFile::VERSION,
    items: Vec::new(),
  };

  library.append_item("koster")?.resize(30)?;
  library.clone_item("koster", "koster_copy")?;

  assert_eq!(library.find_item("koster_copy").map(|item| item.frame_count), Some(30));
  assert!(library.append_item("koster").is_err(), "a name the library holds");
  assert!(library.append_item("").is_err(), "an empty name");
  assert!(library.clone_item("missing", "other").is_err());

  library.rename_item("koster_copy", "lamp")?;

  assert!(
    library.rename_item("lamp", "koster").is_err(),
    "a name another animation holds"
  );
  assert!(
    library.rename_item("lamp", "lamp").is_ok(),
    "an animation keeps its own name"
  );
  assert_eq!(library.remove_item("koster")?.name, "koster");
  assert_eq!(
    library
      .items
      .iter()
      .map(|item| item.name.as_str())
      .collect::<Vec<&str>>(),
    ["lamp"]
  );
  assert!(library.remove_item("koster").is_err());

  Ok(())
}

#[test]
fn normalizes_a_library_as_the_engine_loads_and_saves_it() {
  let mut library: LightAnimFile = LightAnimFile {
    version: LightAnimFile::BGR_VERSION,
    items: vec![LightAnimItem {
      name: String::from("old"),
      fps: 15.0,
      frame_count: 10,
      keys: vec![key(5, 0x80_102030), key(0, 0x00_0000FF), key(5, 0x80_405060)],
    }],
  };

  library.normalize();

  assert_eq!(library.version, LightAnimFile::VERSION);
  assert!(!library.is_bgr());
  assert_eq!(library.items[0].keys, [key(0, 0x00_FF0000), key(5, 0x80_605040)]);
}
