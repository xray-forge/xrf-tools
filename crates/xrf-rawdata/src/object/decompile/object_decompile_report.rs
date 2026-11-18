use serde::Serialize;
use xrf_editable_object::{EditableMesh, EditableSurface};
use xrf_utils::format_path;

use crate::object::decompile::object_decompile::ObjectDecompile;
use crate::object::decompile::object_decompile_request::ObjectDecompileRequest;
use crate::object::decompile::object_inference::ObjectInference;

/// What a compiled model's decompile into an `.object` wrote ([`ObjectDecompiler::decompile_to_path`]).
///
/// [`ObjectDecompiler::decompile_to_path`]: crate::ObjectDecompiler::decompile_to_path
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectDecompileReport {
  pub source: String,
  pub destination: String,
  pub sidecar: String,
  pub surfaces: usize,
  pub points: usize,
  pub faces: usize,
  /// Faces of two-sided surfaces, each folded from a triangle and its mirror.
  pub two_sided_faces: usize,
  pub bones: usize,
  pub inferred: Vec<ObjectInference>,
}

impl ObjectDecompileReport {
  /// The report of `decompile`, written as `request` asked.
  pub(crate) fn new(request: &ObjectDecompileRequest, decompile: &ObjectDecompile) -> Self {
    let mesh: Option<&EditableMesh> = decompile.object.meshes.iter().flatten().next();
    let surfaces: &[EditableSurface] = decompile.object.surfaces.as_deref().unwrap_or_default();

    Self {
      source: format_path(&request.source).to_string(),
      destination: format_path(&request.destination).to_string(),
      sidecar: format_path(&request.get_sidecar_path()).to_string(),
      surfaces: surfaces.len(),
      points: mesh.map_or(0, |mesh| mesh.points.len()),
      faces: mesh.map_or(0, |mesh| mesh.faces.len()),
      two_sided_faces: mesh.map_or(0, |mesh| {
        mesh
          .surface_faces
          .iter()
          .filter(|faces| surfaces.iter().any(|it| it.name == faces.surface && it.is_two_sided()))
          .map(|faces| faces.faces.len())
          .sum()
      }),
      bones: decompile.object.bones.as_ref().map_or(0, Vec::len),
      inferred: decompile.inferred.clone(),
    }
  }
}
