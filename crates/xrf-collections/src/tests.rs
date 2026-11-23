use crate::fast_build_hasher::FastBuildHasher;
use crate::fast_map::FastMap;
use crate::ordered_map::OrderedMap;
use crate::ordered_set::OrderedSet;

#[test]
fn keeps_keys_in_the_order_they_were_first_inserted() {
  let mut map: OrderedMap<&str, u32> = OrderedMap::default();

  map.insert("zeta", 1);
  map.insert("alpha", 2);
  map.insert("zeta", 3);

  assert_eq!(map.iter().collect::<Vec<_>>(), [(&"zeta", &3), (&"alpha", &2)]);

  let set: OrderedSet<&str> = ["b", "a", "b", "c"].into_iter().collect();

  assert_eq!(set.iter().copied().collect::<Vec<_>>(), ["b", "a", "c"]);
}

#[test]
fn builds_with_room_for_what_it_is_told() {
  let map: OrderedMap<u32, u32> = OrderedMap::with_capacity_and_hasher(64, FastBuildHasher);
  let fast: FastMap<u32, u32> = FastMap::with_capacity_and_hasher(64, FastBuildHasher);

  assert!(map.capacity() >= 64);
  assert!(fast.capacity() >= 64);
}
