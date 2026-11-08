use serde::Serialize;

/// How an object compiled compares with an OGF it should stand in for: what differs, by what the engine reads it for,
/// and how much of the geometry matched.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectVerification {
  /// What the model is to the engine: its model type, user data, LODs and motion references.
  pub model: Vec<String>,
  /// Its bones by name, parent and order, and each bone's physics, which scripts, motions and saves key on.
  pub skeleton: Vec<String>,
  /// Its own motions by name and length, and the parameters they play by.
  pub motions: Vec<String>,
  /// What it draws with each texture and shader, over every child drawing with them: the vertices and faces, and the
  /// bounds around them.
  pub geometry: Vec<String>,
  /// The children of each side and the vertices and faces matched between them.
  pub census: ObjectGeometryCensus,
}

/// What the geometry comparison counted.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectGeometryCensus {
  pub children: usize,
  pub reference_children: usize,
  pub vertices: usize,
  /// Compiled vertices one of the reference's stands in for: same position, normal, texture coordinate and weights.
  pub vertices_matched: usize,
  /// Compiled vertices that would match but for their normal: drawn and moved alike, shaded otherwise, as smoothing
  /// read by another rule gives.
  pub vertices_normal_only: usize,
  /// Distinct faces each side draws, a face written twice counted once and its mirror apart.
  pub faces: usize,
  pub reference_faces: usize,
  /// Reference faces the compile draws too, by their corners' positions in winding order.
  pub faces_matched: usize,
  /// Reference faces with two corners at one position, which cover nothing and which a compile drops.
  pub reference_flat_faces: usize,
}

impl ObjectVerification {
  /// Whether nothing the engine reads differs.
  pub fn is_equivalent(&self) -> bool {
    self.model.is_empty() && self.skeleton.is_empty() && self.motions.is_empty() && self.geometry.is_empty()
  }
}
