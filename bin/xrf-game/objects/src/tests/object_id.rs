use crate::id::object_id::ObjectId;

#[test]
fn packs_a_slot_and_a_generation_into_53_bits() {
  let id: ObjectId = ObjectId::new(u32::MAX, ObjectId::MAX_GENERATION).expect("the last generation");

  assert_eq!(id.get_index(), u32::MAX);
  assert_eq!(id.get_generation(), ObjectId::MAX_GENERATION);
  // The largest id is the largest integer a double holds exactly.
  assert_eq!(id.get_raw(), (1 << 53) - 1);
  assert_eq!(ObjectId::new(0, ObjectId::MAX_GENERATION + 1), None);
}

#[test]
fn reads_a_plain_number_back_within_53_bits() {
  let id: ObjectId = ObjectId::new(7, 3).expect("a generation");

  assert_eq!(ObjectId::of_raw(id.get_raw()), Some(id));
  assert_eq!(ObjectId::of_index(12).get_raw(), 12);
  assert_eq!(ObjectId::of_raw(1 << 53), None);
  assert_eq!(id.to_string(), id.get_raw().to_string());
}

#[test]
fn orders_by_slot_before_generation() {
  let old_late: ObjectId = ObjectId::new(9, 0).expect("a generation");
  let new_early: ObjectId = ObjectId::new(2, 5).expect("a generation");

  assert!(new_early < old_late);
  assert!(ObjectId::new(2, 4).expect("a generation") < new_early);
}
