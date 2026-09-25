use serde::Serialize;

use crate::data::details::details_model::DetailsModel;
use crate::data::visual::geometry::visual_section::VisualSection;

/// Everything about a level's packed grass except the bytes themselves.
///
/// The slots and the collision triangles stay in the engine's own space, because the renderer plants them with the
/// engine's own arithmetic and only converts what it planted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailsDescription {
  /// The grid's size in slots, and the world slot its first cell stands for, negated: cell `(x, z)` is world slot
  /// `(x - offset_x, z - offset_z)`.
  pub size_x: u32,
  pub size_z: u32,
  pub offset_x: i32,
  pub offset_z: i32,
  pub models: Vec<DetailsModel>,
  /// One `u32` a cell, `z * size_x + x`: the planted slot's record plus one, zero for a cell with nothing to plant.
  pub grid: VisualSection,
  /// Eight `u32` a planted slot: its stored sixteen bytes as four words, the first entry of its triangle bin, the
  /// bin's length, and its world slot's `x` and `z`.
  pub slots: VisualSection,
  pub slot_count: u32,
  /// One `u32` an entry: the triangle, by index into the triangles.
  pub bins: VisualSection,
  pub bin_length: u32,
  /// Nine floats a triangle: its corners in the engine's space and winding, passable ones left out.
  pub triangles: VisualSection,
  pub triangle_count: u32,
  pub buffer_length: u32,
}
