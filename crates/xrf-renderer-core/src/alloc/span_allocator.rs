use offset_allocator::{Allocation, Allocator};

use crate::alloc::span::Span;
use crate::alloc::span_region::SpanRegion;

/// Hands out and takes back ranges of a growing run of elements. Each growth adds a region of its own allocator, so a
/// range keeps its offset for as long as it is held.
pub struct SpanAllocator {
  regions: Vec<SpanRegion>,
  capacity: u32,
}

impl SpanAllocator {
  /// Allocations a region tracks at most.
  pub const MAX_ALLOCATIONS: u32 = 128 * 1024;

  pub fn new(capacity: u32) -> Self {
    let mut allocator: Self = Self {
      regions: Vec::new(),
      capacity: 0,
    };

    allocator.grow(capacity.max(1));
    allocator
  }

  /// Elements covered, held or free.
  pub fn get_capacity(&self) -> u32 {
    self.capacity
  }

  /// Elements free, over every region, however scattered.
  pub fn get_free(&self) -> u32 {
    self
      .regions
      .iter()
      .map(|region| region.allocator.storage_report().total_free_space)
      .sum()
  }

  /// A range of `count` elements from the first region with room, or `None` where none has.
  pub fn allocate(&mut self, count: u32) -> Option<Span> {
    let count: u32 = count.max(1);

    self.regions.iter_mut().enumerate().find_map(|(index, region)| {
      let allocation: Allocation = region.allocator.allocate(count)?;

      Some(Span {
        offset: region.base + allocation.offset,
        count,
        region: index as u32,
        allocation,
      })
    })
  }

  /// The capacity growing to fit `count` more elements reaches: at least double, and a new region of at least twice
  /// `count`, since the allocator rounds a request up to its size bin and a region exactly the request's size may not
  /// hold it.
  pub fn get_grown_capacity(&self, count: u32) -> u32 {
    self
      .capacity
      .saturating_mul(2)
      .max(self.capacity.saturating_add(count.max(1).saturating_mul(2)))
  }

  /// Covers `capacity` elements, the new ones a region of their own.
  ///
  /// # Panics
  ///
  /// When `capacity` is not more than is covered already.
  pub fn grow(&mut self, capacity: u32) {
    assert!(capacity > self.capacity, "A span allocator only grows");

    let added: u32 = capacity - self.capacity;

    self.regions.push(SpanRegion {
      allocator: Allocator::with_max_allocs(added, Self::MAX_ALLOCATIONS.min(added.max(2))),
      base: self.capacity,
      capacity: added,
    });
    self.capacity = capacity;
  }

  pub fn free(&mut self, span: Span) {
    self.regions[span.region as usize].allocator.free(span.allocation);
  }
}
