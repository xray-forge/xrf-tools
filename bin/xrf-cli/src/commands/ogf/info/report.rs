use serde::Serialize;
use xrf_ogf::OgfFile;

use crate::commands::ogf::info::ogf_bone_report::OgfBoneReport;
use crate::commands::ogf::info::ogf_bounds_report::OgfBoundsReport;
use crate::commands::ogf::info::ogf_description_report::OgfDescriptionReport;
use crate::commands::ogf::info::ogf_material_report::OgfMaterialReport;
use crate::commands::ogf::info::ogf_residue_report::OgfResidueReport;

/// What `ogf info` read out of a visual.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfInfoReport {
  bones: Vec<OgfBoneReport>,
  bounds: OgfBoundsReport,
  children: Vec<OgfMaterialReport>,
  description: Option<OgfDescriptionReport>,
  material: Option<OgfMaterialReport>,
  model_type: u8,
  motion_refs: Vec<String>,
  progressive_lods: usize,
  residue: Option<OgfResidueReport>,
  shader_id: u16,
  unknown_chunks: Option<Vec<u32>>,
  version: u8,
}

impl OgfInfoReport {
  pub fn new(file: &OgfFile, unknown_chunks: Option<Vec<u32>>) -> Self {
    Self {
      bones: file
        .bones
        .as_ref()
        .map_or_else(Vec::new, |bones| bones.bones.iter().map(OgfBoneReport::new).collect()),
      bounds: OgfBoundsReport::new(&file.header.bounding_box, &file.header.bounding_sphere),
      children: file.children.as_ref().map_or_else(Vec::new, |children| {
        children
          .nested
          .iter()
          .filter_map(|child| child.texture.as_ref())
          .map(OgfMaterialReport::new)
          .collect()
      }),
      description: file.get_description().map(OgfDescriptionReport::new),
      material: file.texture.as_ref().map(OgfMaterialReport::new),
      model_type: file.header.model_type,
      motion_refs: file
        .kinematics
        .as_ref()
        .map_or_else(Vec::new, |kinematics| kinematics.motion_refs.clone()),
      progressive_lods: file.swi_data.as_ref().map_or(0, |swi| swi.windows.len()),
      residue: file.residue.as_ref().map(|residue| {
        OgfResidueReport::new(
          residue,
          file
            .kinematics
            .as_ref()
            .map_or(0, |kinematics| kinematics.trailing.len()),
        )
      }),
      shader_id: file.header.shader_id,
      unknown_chunks,
      version: file.header.version,
    }
  }
}
