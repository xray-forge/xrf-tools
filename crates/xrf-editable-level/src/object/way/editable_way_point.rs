use xrf_math::Vector3d;

use crate::object::way::editable_way_link::EditableWayLink;

/// One point of a way, `CWayPoint`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableWayPoint {
  /// Unique within its way, `wp00` upwards for a point the editor adds.
  pub name: String,
  pub position: Vector3d,
  /// Thirty-two flags the scripts give meaning to.
  pub flags: u32,
  pub selected: bool,
  /// Its links in the order the editor keeps them.
  pub links: Vec<EditableWayLink>,
}
