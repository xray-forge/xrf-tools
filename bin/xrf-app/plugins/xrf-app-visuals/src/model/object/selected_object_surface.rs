use serde::Serialize;
use xrf_editable_object::{EditableMesh, EditableSurface};

/// One surface of an object as it was authored, with what its OGF keeps only part of: the compiler shader and game
/// material the export drops, and whether it is drawn from both sides.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedObjectSurface {
  pub name: String,
  pub texture: String,
  /// The engine shader, which the OGF keeps.
  pub shader: String,
  /// The shader the level compiler lights it by, which the OGF does not keep.
  pub compiler_shader: String,
  /// What it sounds and breaks like, which the OGF does not keep.
  pub game_material: String,
  pub is_two_sided: bool,
  /// Faces drawn with it, over every mesh.
  pub face_count: u32,
}

impl SelectedObjectSurface {
  /// The surface, its faces counted over `meshes`.
  pub fn of(surface: &EditableSurface, meshes: &[EditableMesh]) -> Self {
    let face_count: usize = meshes
      .iter()
      .flat_map(|mesh| &mesh.surface_faces)
      .filter(|faces| faces.surface == surface.name)
      .map(|faces| faces.faces.len())
      .sum();

    Self {
      name: surface.name.clone(),
      texture: surface.texture.clone(),
      shader: surface.shader.clone(),
      compiler_shader: surface.compiler_shader.clone(),
      game_material: surface.game_material.clone(),
      is_two_sided: surface.is_two_sided(),
      face_count: u32::try_from(face_count).unwrap_or(u32::MAX),
    }
  }
}
