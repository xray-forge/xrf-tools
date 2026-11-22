use xrf_math::glam::{Vec3, Vec3A};

use crate::tracer::xrlc::xrlc_child::XrlcChild;

/// One node of xrLC's ray model (`AABBNoLeafNode`): the box of every face below it and its two sides, the first
/// (`GetPos`) walked before the second (`GetNeg`).
///
/// The box is held as `ray_collider::_box_sse` gives back Opcode's centre and half extents, rounded in single
/// precision, so it can end a rounding short of the faces it holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct XrlcNode {
  pub(crate) low: Vec3A,
  pub(crate) high: Vec3A,
  pub(crate) children: [XrlcChild; 2],
}

impl XrlcNode {
  /// The node of the box from `low` to `high`, kept as Opcode keeps it (`AABB::GetCenter`, `AABB::GetExtents`).
  pub(crate) fn new(low: Vec3, high: Vec3, children: [XrlcChild; 2]) -> Self {
    let center: Vec3A = Vec3A::from((high + low) * 0.5);
    let extents: Vec3A = Vec3A::from((high - low) * 0.5);

    Self {
      low: center - extents,
      high: center + extents,
      children,
    }
  }

  /// Where along a ray from `origin` with the inverse direction `inverse` its line enters the box, where the ray
  /// reaches it at all: `isect_sse`, lane for lane. The entry may lie behind the origin.
  ///
  /// A ray along a box's face gives `0 * inf` on that axis, which the SSE clamps to infinity, so the axis bounds
  /// nothing.
  pub(crate) fn get_entry(&self, origin: Vec3A, inverse: Vec3A) -> Option<f32> {
    let first: Vec3A = (self.low - origin) * inverse;
    let second: Vec3A = (self.high - origin) * inverse;
    let far: Vec3A = Self::max_ps(
      Self::min_ps(first, Vec3A::INFINITY),
      Self::min_ps(second, Vec3A::INFINITY),
    );
    let near: Vec3A = Self::min_ps(
      Self::max_ps(first, Vec3A::NEG_INFINITY),
      Self::max_ps(second, Vec3A::NEG_INFINITY),
    );
    // The lanes folded as the shuffles fold them: `x` with `y`, then with `z`.
    let far: f32 = Self::min_ss(Self::min_ss(far.x, far.y), far.z);
    let near: f32 = Self::max_ss(Self::max_ss(near.x, near.y), near.z);

    (far >= 0.0 && far >= near).then_some(near)
  }

  /// `_mm_min_ps`: `first` where it is less, else `second`.
  fn min_ps(first: Vec3A, second: Vec3A) -> Vec3A {
    Vec3A::select(first.cmplt(second), first, second)
  }

  /// `_mm_max_ps`: `first` where it is greater, else `second`.
  fn max_ps(first: Vec3A, second: Vec3A) -> Vec3A {
    Vec3A::select(first.cmpgt(second), first, second)
  }

  /// `_mm_min_ss` on the first lane.
  fn min_ss(first: f32, second: f32) -> f32 {
    if first < second { first } else { second }
  }

  /// `_mm_max_ss` on the first lane.
  fn max_ss(first: f32, second: f32) -> f32 {
    if first > second { first } else { second }
  }
}
