use crate::id::id_map::IdMap;
use crate::id::object_id::ObjectId;

#[test]
fn holds_values_by_id_and_iterates_in_id_order_whatever_order_they_came_in() {
  let mut table: IdMap<&str> = IdMap::default();

  assert_eq!(table.insert(ObjectId(40), "forty"), None);
  assert_eq!(table.insert(ObjectId(3), "three"), None);
  assert_eq!(table.insert(ObjectId(17), "seventeen"), None);
  assert_eq!(table.insert(ObjectId(3), "three again"), Some("three"));

  assert_eq!(table.len(), 3);
  assert_eq!(table.get(ObjectId(17)), Some(&"seventeen"));
  assert_eq!(table.get(ObjectId(18)), None);
  assert_eq!(table.get(ObjectId(60_000)), None);
  assert_eq!(
    table.iter().map(|(id, value)| (id.0, *value)).collect::<Vec<_>>(),
    vec![(3, "three again"), (17, "seventeen"), (40, "forty")]
  );
}

#[test]
fn takes_a_value_out_and_counts_what_is_left() {
  let mut table: IdMap<u32> = IdMap::default();

  table.insert(ObjectId(1), 10);
  table.insert(ObjectId(2), 20);

  assert_eq!(table.remove(ObjectId(1)), Some(10));
  assert_eq!(table.remove(ObjectId(1)), None);
  assert_eq!(table.remove(ObjectId(9)), None);
  assert_eq!(table.len(), 1);
  assert!(!table.contains(ObjectId(1)));

  for (_, value) in table.iter_mut() {
    *value += 1;
  }

  assert_eq!(table.get(ObjectId(2)), Some(&21));
}

#[test]
fn reads_xrays_none_as_no_id() {
  assert_eq!(ObjectId::of(0xffff), None);
  assert_eq!(ObjectId::of(0), Some(ObjectId(0)));
}
