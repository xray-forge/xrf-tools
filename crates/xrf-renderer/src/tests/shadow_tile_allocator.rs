use crate::scene::level::shadow_tile::ShadowTile;
use crate::scene::level::shadow_tile_allocator::ShadowTileAllocator;

#[test]
fn shadow_tile_allocator_fills_the_atlas_with_squares_that_never_overlap() {
  let mut allocator: ShadowTileAllocator = ShadowTileAllocator::new(4096, 32);
  let tiles: Vec<ShadowTile> = (0..16).map_while(|_| allocator.allocate(1024)).collect();

  assert_eq!(tiles.len(), 16);
  assert_eq!(allocator.allocate(32), None);
  assert_eq!(allocator.get_used(), 4096 * 4096);

  for (index, a) in tiles.iter().enumerate() {
    for b in &tiles[index + 1..] {
      let is_apart: bool = a.x + a.size <= b.x || b.x + b.size <= a.x || a.y + a.size <= b.y || b.y + b.size <= a.y;

      assert!(is_apart, "{a:?} {b:?}");
    }
  }
}

#[test]
fn shadow_tile_allocator_rounds_up_and_joins_released_squares() {
  let mut allocator: ShadowTileAllocator = ShadowTileAllocator::new(4096, 32);
  let small: ShadowTile = allocator.allocate(20).unwrap();

  assert_eq!(small.size, 32);
  assert_eq!(allocator.allocate(700).unwrap().size, 1024);

  let tiles: Vec<ShadowTile> = (0..3).map(|_| allocator.allocate(32).unwrap()).collect();

  allocator.release(small);
  tiles.into_iter().for_each(|tile| allocator.release(tile));

  let whole: Vec<ShadowTile> = std::iter::from_fn(|| allocator.allocate(1024)).collect();

  // The four 32s joined back up, so every 1024 but the one held is whole again.
  assert_eq!(whole.len(), 15);
}
