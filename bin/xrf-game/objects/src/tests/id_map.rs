use crate::id::id_map::IdMap;
use crate::id::object_id::ObjectId;

fn id(index: u32) -> ObjectId {
  ObjectId::of_index(index)
}

#[test]
fn holds_values_by_id_and_iterates_in_id_order_whatever_order_they_came_in() {
  let mut table: IdMap<&str> = IdMap::default();

  assert_eq!(table.insert(id(40), "forty"), None);
  assert_eq!(table.insert(id(3), "three"), None);
  assert_eq!(table.insert(id(17), "seventeen"), None);
  assert_eq!(table.insert(id(3), "three again"), Some("three"));

  assert_eq!(table.len(), 3);
  assert_eq!(table.get(id(17)), Some(&"seventeen"));
  assert_eq!(table.get(id(18)), None);
  assert_eq!(table.get(id(60_000)), None);
  assert_eq!(
    table
      .iter()
      .map(|(id, value)| (id.get_index(), *value))
      .collect::<Vec<_>>(),
    vec![(3, "three again"), (17, "seventeen"), (40, "forty")]
  );
}

#[test]
fn takes_a_value_out_and_counts_what_is_left() {
  let mut table: IdMap<u32> = IdMap::default();

  table.insert(id(1), 10);
  table.insert(id(2), 20);

  assert_eq!(table.remove(id(1)), Some(10));
  assert_eq!(table.remove(id(1)), None);
  assert_eq!(table.remove(id(9)), None);
  assert_eq!(table.len(), 1);
  assert!(!table.contains(id(1)));

  for (_, value) in table.iter_mut() {
    *value += 1;
  }

  assert_eq!(table.get(id(2)), Some(&21));
}

#[test]
fn finds_nothing_for_an_id_of_a_slots_other_generation() {
  let mut table: IdMap<&str> = IdMap::default();
  let first: ObjectId = id(4);
  let second: ObjectId = ObjectId::new(4, 1).expect("a generation");

  table.insert(first, "first");

  assert_eq!(table.get(second), None);
  assert_eq!(table.remove(second), None);

  // A row the slot's earlier object left is replaced, never answered as the newcomer's.
  assert_eq!(table.insert(second, "second"), None);
  assert_eq!(table.get(first), None);
  assert_eq!(table.get(second), Some(&"second"));
  assert_eq!(table.len(), 1);
  assert_eq!(table.iter().map(|(id, _)| id).collect::<Vec<_>>(), vec![second]);
}

#[test]
fn iterates_from_an_id_on_whether_or_not_it_holds_one() {
  let mut table: IdMap<u32> = IdMap::default();

  for index in [2, 5, 9] {
    table.insert(id(index), index);
  }

  let ids = |from: u32| {
    table
      .iter_from(id(from))
      .map(|(id, _)| id.get_index())
      .collect::<Vec<_>>()
  };

  assert_eq!(ids(0), vec![2, 5, 9]);
  assert_eq!(ids(5), vec![5, 9]);
  assert_eq!(ids(6), vec![9]);
  assert_eq!(ids(10), Vec::<u32>::new());
  assert_eq!(ids(u32::MAX), Vec::<u32>::new());
}
