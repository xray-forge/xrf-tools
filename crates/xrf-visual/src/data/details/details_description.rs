use serde::Serialize;

use crate::data::details::details_model::DetailsModel;
use crate::data::visual::geometry::visual_section::VisualSection;

/// Everything about a level's packed grass except the bytes themselves.
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
  /// Six `u32` a planted slot: its stored sixteen bytes as four words, the first entry of its triangle bin, and the
  /// bin's length. Its world slot is its cell's, which the grid says.
  pub slots: VisualSection,
  pub slot_count: u32,
  /// One `u32` an entry: the triangle, by index into the triangles.
  pub bins: VisualSection,
  /// Nine floats a triangle: its corners in renderer space, wound for it, passable ones left out.
  pub triangles: VisualSection,
  pub buffer_length: u32,
}

impl DetailsDescription {
  /// Bytes one bin entry takes.
  pub const BIN_ENTRY_BYTES: u32 = 4;

  /// Bytes one triangle takes: nine floats.
  pub const TRIANGLE_BYTES: u32 = 9 * 4;

  /// Entries the bins hold, all of them together.
  pub fn get_bin_length(&self) -> u32 {
    self.bins.byte_length / Self::BIN_ENTRY_BYTES
  }

  /// Triangles the plantings are cast onto.
  pub fn get_triangle_count(&self) -> u32 {
    self.triangles.byte_length / Self::TRIANGLE_BYTES
  }
}
