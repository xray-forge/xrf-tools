/// One node of the tracer's hierarchy: the box around its triangles, and where they or its children are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelCformTracerNode {
  pub(crate) min: [f32; 3],
  pub(crate) max: [f32; 3],
  /// A leaf's first triangle, or an inner node's second child; its first child follows it.
  pub(crate) start: u32,
  /// A leaf's triangle count, zero for an inner node.
  pub(crate) count: u32,
}

impl LevelCformTracerNode {
  /// A leaf over a run of triangles, its box empty until it is bounded.
  pub(crate) fn leaf(start: u32, count: u32) -> Self {
    Self {
      count,
      max: [f32::NEG_INFINITY; 3],
      min: [f32::INFINITY; 3],
      start,
    }
  }

  /// An inner node, told its second child and bounded once they are laid out.
  pub(crate) fn inner() -> Self {
    Self::leaf(0, 0)
  }

  /// Widens the box to hold another.
  pub(crate) fn enclose(&mut self, min: &[f32; 3], max: &[f32; 3]) {
    for axis in 0..3 {
      self.min[axis] = self.min[axis].min(min[axis]);
      self.max[axis] = self.max[axis].max(max[axis]);
    }
  }

  /// Whether a ray reaches the box before `range`, by the slabs of each axis.
  pub(crate) fn is_reached(&self, origin: &[f32; 3], inverse: &[f32; 3], range: f32) -> bool {
    let mut near: f32 = 0.0;
    let mut far: f32 = range;

    for axis in 0..3 {
      // A ray along the slab's planes meets it everywhere or nowhere, as its origin stands; the products would give NaN
      // for an origin on a plane, which `min` and `max` drop for the infinity beside it, refusing a box the ray is on.
      if inverse[axis].is_infinite() {
        if origin[axis] < self.min[axis] || origin[axis] > self.max[axis] {
          return false;
        }

        continue;
      }

      let first: f32 = (self.min[axis] - origin[axis]) * inverse[axis];
      let second: f32 = (self.max[axis] - origin[axis]) * inverse[axis];

      near = near.max(first.min(second));
      far = far.min(first.max(second));
    }

    near <= far
  }
}
