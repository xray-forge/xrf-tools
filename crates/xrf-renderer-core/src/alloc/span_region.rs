use offset_allocator::Allocator;

/// One stretch of a span buffer, as it grew: its own allocator over `capacity` elements from `base`.
pub(crate) struct SpanRegion {
  pub allocator: Allocator,
  pub base: u32,
  pub capacity: u32,
}
