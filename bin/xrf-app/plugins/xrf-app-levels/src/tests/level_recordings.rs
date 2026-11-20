use crate::viewer::recording::LevelRecordings;

// A stop or a window's release holding the slot is no running recording; a start's check under the lock decides.
#[test]
fn reads_a_held_empty_slot_as_no_recording() {
  let recordings: LevelRecordings = LevelRecordings::default();
  let held = recordings.lock();

  assert!(held.is_none());
  assert!(!recordings.is_recording());
}
