use crate::alloc::{Span, SpanAllocator};

#[test]
fn reuses_a_freed_range() {
  let mut allocator: SpanAllocator = SpanAllocator::new(1024);
  let first: Span = allocator.allocate(100).unwrap();
  let offset: u32 = first.get_offset();

  allocator.free(first);

  assert_eq!(allocator.allocate(100).unwrap().get_offset(), offset);
}

#[test]
fn refuses_what_no_region_has_room_for() {
  let mut allocator: SpanAllocator = SpanAllocator::new(64);

  assert!(allocator.allocate(65).is_none());
  assert!(allocator.allocate(64).is_some());
  assert!(allocator.allocate(1).is_none());
}

#[test]
fn grows_into_a_new_region_keeping_held_offsets() {
  let mut allocator: SpanAllocator = SpanAllocator::new(64);
  let held: Span = allocator.allocate(64).unwrap();
  let capacity: u32 = allocator.get_grown_capacity(100);

  assert_eq!(capacity, 264);

  allocator.grow(capacity);

  let added: Span = allocator.allocate(100).unwrap();

  assert_eq!(held.get_offset(), 0);
  assert!(added.get_offset() >= 64, "the new range lies in the new region");
  assert_eq!(allocator.get_capacity(), 264);

  allocator.free(held);

  assert_eq!(
    allocator.allocate(64).unwrap().get_offset(),
    0,
    "a freed range of the first region is reused"
  );
}

#[test]
fn grows_at_least_double() {
  let allocator: SpanAllocator = SpanAllocator::new(1000);

  assert_eq!(allocator.get_grown_capacity(1), 2000);
}
