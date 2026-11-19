use crate::object::verify::object_geometry_census::ObjectGeometryCensus;
use crate::object::verify::object_verification_durations::ObjectVerificationDurations;

/// How an object compiled compares with an OGF it should stand in for: what differs, by what the engine reads it for,
/// how much of the geometry matched, and how long each area took.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
  /// How long each area's comparison took.
  pub durations: ObjectVerificationDurations,
}

impl ObjectVerification {
  /// Whether nothing the engine reads differs.
  pub fn is_equivalent(&self) -> bool {
    self.model.is_empty() && self.skeleton.is_empty() && self.motions.is_empty() && self.geometry.is_empty()
  }
}
