use std::collections::BTreeSet;

use serde::Serialize;
use xrf_editable_object::{EditableMesh, EditableObjectFile};
use xrf_rawdata::ObjectProvenance;

use crate::model::object::selected_object_bone::SelectedObjectBone;
use crate::model::object::selected_object_provenance::SelectedObjectProvenance;
use crate::model::object::selected_object_surface::SelectedObjectSurface;

/// What an SDK object holds that the model compiled from it does not show: its surfaces and bones as authored, its
/// mesh's points and smoothing, and where a decompiled one came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedObjectDescription {
  pub surfaces: Vec<SelectedObjectSurface>,
  pub bones: Vec<SelectedObjectBone>,
  pub mesh_count: u32,
  /// Points over every mesh, before the export splits them at seams and hard edges.
  pub point_count: u32,
  pub face_count: u32,
  /// Distinct smoothing values over every face, `0` where no mesh stores smoothing.
  pub smoothing_group_count: u32,
  pub motion_references: Vec<String>,
  /// Whether the object carries motions of its own, which the compile writes into the OGF.
  pub has_embedded_motions: bool,
  pub class_script: Option<String>,
  pub lods: Option<String>,
  /// Where a decompiled object came from, or none for an authored one.
  pub provenance: Option<SelectedObjectProvenance>,
}

impl SelectedObjectDescription {
  /// The description of `object`, with its provenance where its sidecar holds one.
  pub fn of(object: &EditableObjectFile, provenance: Option<&ObjectProvenance>) -> Self {
    let meshes: &[EditableMesh] = object.meshes.as_deref().unwrap_or_default();
    let count = |value: usize| u32::try_from(value).unwrap_or(u32::MAX);
    let smoothing: BTreeSet<u32> = meshes
      .iter()
      .flat_map(|mesh| mesh.smoothing_groups.iter().flatten())
      .copied()
      .collect();

    Self {
      surfaces: object
        .surfaces
        .iter()
        .flatten()
        .map(|surface| SelectedObjectSurface::of(surface, meshes))
        .collect(),
      bones: object.bones.iter().flatten().map(SelectedObjectBone::of).collect(),
      mesh_count: count(meshes.len()),
      point_count: count(meshes.iter().map(|mesh| mesh.points.len()).sum()),
      face_count: count(meshes.iter().map(|mesh| mesh.faces.len()).sum()),
      smoothing_group_count: count(smoothing.len()),
      motion_references: object.motion_references.clone().unwrap_or_default(),
      has_embedded_motions: object
        .embedded_motions
        .as_ref()
        .is_some_and(|motions| !motions.is_empty()),
      class_script: object.class_script.clone().filter(|script| !script.is_empty()),
      lods: object.lods.clone().filter(|lods| !lods.is_empty()),
      provenance: provenance.map(SelectedObjectProvenance::from),
    }
  }
}
