//! The renderer's GPU allocators: span buffers for what persists and changes, an upload ring for what one frame needs.

mod span;
mod span_allocator;
mod span_buffer;
mod span_lane;
mod span_region;
mod span_writes;
mod upload_ring;
mod upload_slice;

#[cfg(test)]
mod tests;

pub use span::Span;
pub use span_allocator::SpanAllocator;
pub use span_buffer::SpanBuffer;
pub use span_lane::SpanLane;
pub use span_writes::SpanWrites;
pub use upload_ring::UploadRing;
pub use upload_slice::UploadSlice;
