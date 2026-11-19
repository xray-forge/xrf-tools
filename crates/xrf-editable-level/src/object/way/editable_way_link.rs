/// A link from one way point to another of its way, `SWPLink`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableWayLink {
  /// The point it leads to, by its place in the way's points.
  pub to: usize,
  pub probability: f32,
}
