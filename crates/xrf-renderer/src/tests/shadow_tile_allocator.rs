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

// The audit of its arithmetic (2026-10-17): whatever the order squares are taken and given back in, the live ones never
// overlap or leave the atlas, its count of texels used is theirs, and once all are back the atlas is one square again.
#[test]
fn shadow_tile_allocator_keeps_its_squares_apart_through_any_order_of_takes_and_returns() {
  const SIZE: u32 = 2048;
  let mut allocator: ShadowTileAllocator = ShadowTileAllocator::new(SIZE, 32);
  let mut live: Vec<ShadowTile> = Vec::new();
  // A linear congruential generator, so the sequence is the same every run.
  let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
  let mut next = |bound: u64| -> u64 {
    seed = seed
      .wrapping_mul(6_364_136_223_846_793_005)
      .wrapping_add(1_442_695_040_888_963_407);

    (seed >> 33) % bound
  };

  for _ in 0..4000 {
    if live.is_empty() || next(3) > 0 {
      if let Some(tile) = allocator.allocate(16 << next(6)) {
        live.push(tile);
      }
    } else {
      let index: usize = next(live.len() as u64) as usize;

      allocator.release(live.swap_remove(index));
    }

    let used: u64 = live.iter().map(|tile| (tile.size as u64) * (tile.size as u64)).sum();

    assert_eq!(allocator.get_used(), used);

    for (index, a) in live.iter().enumerate() {
      assert!(a.x + a.size <= SIZE && a.y + a.size <= SIZE, "{a:?} leaves the atlas");
      assert!(
        a.size.is_power_of_two() && a.x % a.size == 0 && a.y % a.size == 0,
        "{a:?} is not aligned"
      );

      for b in &live[index + 1..] {
        let is_apart: bool = a.x + a.size <= b.x || b.x + b.size <= a.x || a.y + a.size <= b.y || b.y + b.size <= a.y;

        assert!(is_apart, "{a:?} {b:?}");
      }
    }
  }

  live.drain(..).for_each(|tile| allocator.release(tile));

  assert_eq!(allocator.get_used(), 0);
  assert_eq!(allocator.allocate(SIZE), Some(ShadowTile { x: 0, y: 0, size: SIZE }));
}

#[test]
fn shadow_tile_allocator_refuses_a_square_larger_than_the_atlas_and_gives_the_atlas_whole() {
  let mut allocator: ShadowTileAllocator = ShadowTileAllocator::new(1024, 32);

  assert_eq!(allocator.allocate(2048), None);
  assert_eq!(allocator.get_used(), 0);

  let whole: ShadowTile = allocator.allocate(1024).expect("the atlas whole");

  assert_eq!((whole.x, whole.y, whole.size), (0, 0, 1024));
  assert_eq!(allocator.allocate(32), None);
}
