/// One of the parallel arrays a span buffer holds: element `i` of every lane belongs to the same item, so one range
/// addresses them all.
#[derive(Clone, Copy, Debug)]
pub struct SpanLane {
  pub label: &'static str,
  /// Bytes an element of this lane takes.
  pub stride: u64,
}

impl SpanLane {
  pub const fn new(label: &'static str, stride: u64) -> Self {
    Self { label, stride }
  }
}
