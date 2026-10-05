use crate::store::{ProxyHandle, ProxyStore};

#[test]
fn reaches_each_item_by_its_handle() {
  let mut store: ProxyStore<&str> = ProxyStore::new();
  let first: ProxyHandle<&str> = store.add("first");
  let second: ProxyHandle<&str> = store.add("second");

  assert_eq!(store.len(), 2);
  assert_eq!(store.get(first), Some(&"first"));
  assert_eq!(store.get(second), Some(&"second"));
  assert_eq!((store.get_index(first), store.get_index(second)), (Some(0), Some(1)));
}

#[test]
fn moves_the_last_record_into_a_removed_one_and_keeps_its_handle() {
  let mut store: ProxyStore<u32> = ProxyStore::new();
  let handles: Vec<ProxyHandle<u32>> = (0..4).map(|value| store.add(value)).collect();

  store.take_changed();

  assert_eq!(store.remove(handles[1]), Some(1));
  assert_eq!(store.as_slice(), &[0, 3, 2]);
  assert_eq!(store.get(handles[3]), Some(&3));
  assert_eq!(store.get_index(handles[3]), Some(1));
  assert_eq!(store.take_changed(), [1], "the moved record goes up again");
}

#[test]
fn answers_nothing_for_a_removed_item_even_once_its_slot_is_reused() {
  let mut store: ProxyStore<u32> = ProxyStore::new();
  let gone: ProxyHandle<u32> = store.add(1);

  store.remove(gone);

  let reused: ProxyHandle<u32> = store.add(2);

  assert_eq!(store.get(gone), None);
  assert_eq!(store.remove(gone), None);
  assert_eq!(store.get(reused), Some(&2));
  assert_ne!(gone, reused, "the reused slot's handle carries a newer generation");
}

#[test]
fn notes_each_record_added_or_changed_once() {
  let mut store: ProxyStore<u32> = ProxyStore::new();
  let first: ProxyHandle<u32> = store.add(1);
  let second: ProxyHandle<u32> = store.add(2);

  assert_eq!(store.take_changed(), [0, 1]);
  assert!(store.take_changed().is_empty());

  *store.get_mut(second).unwrap() = 20;
  *store.get_mut(second).unwrap() = 21;

  assert_eq!(store.take_changed(), [1]);

  store.remove(first);

  assert_eq!(store.take_changed(), [0], "the last record moved into the first's place");

  store.remove(second);

  assert!(store.take_changed().is_empty(), "nothing stands to go up");
}

#[test]
fn walks_the_records_with_their_handles() {
  let mut store: ProxyStore<char> = ProxyStore::new();
  let handles: Vec<ProxyHandle<char>> = ['a', 'b', 'c'].into_iter().map(|item| store.add(item)).collect();

  store.remove(handles[0]);

  let walked: Vec<(ProxyHandle<char>, char)> = store.iter().map(|(handle, item)| (handle, *item)).collect();

  assert_eq!(walked, [(handles[2], 'c'), (handles[1], 'b')]);
}
