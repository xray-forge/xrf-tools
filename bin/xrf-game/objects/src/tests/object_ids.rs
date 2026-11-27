use crate::id::object_id::ObjectId;
use crate::id::object_ids::ObjectIds;

#[test]
fn hands_out_freed_slots_a_generation_on_before_new_ones() {
  let mut ids: ObjectIds = ObjectIds::default();

  let first: Option<ObjectId> = ids.take();
  let second: Option<ObjectId> = ids.take();

  assert_eq!(first, Some(ObjectId::of_index(0)));
  assert_eq!(second, Some(ObjectId::of_index(1)));

  ids.release(ObjectId::of_index(1));
  ids.release(ObjectId::of_index(0));

  // The lowest freed slot first, whatever order they were freed in.
  assert_eq!(ids.take(), ObjectId::new(0, 1));
  assert_eq!(ids.take(), ObjectId::new(1, 1));
  assert_eq!(ids.take(), Some(ObjectId::of_index(2)));
}

#[test]
fn retires_a_slot_freed_at_its_last_generation() {
  let mut ids: ObjectIds = ObjectIds::default();

  ids.reserve(ObjectId::of_index(3));
  ids.release(ObjectId::new(3, ObjectId::MAX_GENERATION).expect("the last generation"));

  assert_eq!(ids.take(), Some(ObjectId::of_index(4)));
}

#[test]
fn creates_past_a_reserved_id_and_never_hands_it_out() {
  let mut ids: ObjectIds = ObjectIds::default();

  ids.release(ObjectId::of_index(5));
  ids.reserve(ObjectId::new(5, 1).expect("a generation"));

  assert_eq!(ids.take(), Some(ObjectId::of_index(6)));
}
