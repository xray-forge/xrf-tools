use serde::Serialize;
use xrf_thm::ThmMaterialChunk;

/// The shading declaration of a descriptor, `THM_CHUNK_MATERIAL`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmMaterial {
  /// The two lighting models the surface sits between.
  pub label: String,
  pub value: u32,
  /// Where between them it sits.
  pub weight: f32,
}

impl ArchiveThmMaterial {
  pub fn of(material: &ThmMaterialChunk) -> Self {
    Self {
      label: material.material.label(),
      value: material.material.into(),
      weight: material.weight,
    }
  }
}
