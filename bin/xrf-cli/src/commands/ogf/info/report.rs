use serde::Serialize;
use xrf_db::{OgfBox, OgfFile, OgfSphere};

/// A bounding volume, reported as the numbers the file carries rather than as debug text.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfBoundsReport {
  maximum: [f32; 3],
  minimum: [f32; 3],
  radius: f32,
  sphere: [f32; 3],
}

impl OgfBoundsReport {
  fn new(bounding_box: &OgfBox, bounding_sphere: &OgfSphere) -> Self {
    Self {
      maximum: [bounding_box.max.x, bounding_box.max.y, bounding_box.max.z],
      minimum: [bounding_box.min.x, bounding_box.min.y, bounding_box.min.z],
      radius: bounding_sphere.radius,
      sphere: [
        bounding_sphere.position.x,
        bounding_sphere.position.y,
        bounding_sphere.position.z,
      ],
    }
  }
}

/// The texture and shader a visual or one of its children is dressed with.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfMaterialReport {
  shader: String,
  texture: String,
}

/// One bone and the bone it hangs from.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfBoneReport {
  name: String,
  parent: String,
}

/// Bytes a visual ends with that the engine's loader never reads.
///
/// Present only for a malformed visual the engine loads anyway. A patch discards these bytes, so this is the record
/// that they were there and what they held.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfResidueReport {
  cause: String,
  /// The motion reference path the residue completes, for the split-reference shape only.
  discarded_reference: Option<String>,
  position: u64,
  /// Every ignored byte, counting those inside the motion refs chunk as well as those after it.
  size: usize,
}

/// What `ogf info` read out of a visual.
///
/// Everything the command tells a human, including the per-bone and per-child detail it only prints
/// in full at higher verbosity: a machine consumer has no `--verbose` to raise. `unknownChunks`
/// carries the chunk ids the reader did not parse, which is how an unsupported visual is told apart
/// from an empty one; it is absent when the survey itself could not run.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfInfoReport {
  bones: Vec<OgfBoneReport>,
  bounds: OgfBoundsReport,
  children: Vec<OgfMaterialReport>,
  description: Option<String>,
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
      bones: file.bones.as_ref().map_or_else(Vec::new, |bones| {
        bones
          .bones
          .iter()
          .map(|bone| OgfBoneReport {
            name: bone.name.clone(),
            parent: bone.parent.clone(),
          })
          .collect()
      }),
      bounds: OgfBoundsReport::new(&file.header.bounding_box, &file.header.bounding_sphere),
      children: file.children.as_ref().map_or_else(Vec::new, |children| {
        children
          .nested
          .iter()
          .filter_map(|child| child.texture.as_ref())
          .map(|texture| OgfMaterialReport {
            shader: texture.shader_name.clone(),
            texture: texture.texture_name.clone(),
          })
          .collect()
      }),
      description: file.description.as_ref().map(|description| format!("{description:?}")),
      material: file.texture.as_ref().map(|texture| OgfMaterialReport {
        shader: texture.shader_name.clone(),
        texture: texture.texture_name.clone(),
      }),
      model_type: file.header.model_type,
      motion_refs: file
        .kinematics
        .as_ref()
        .map_or_else(Vec::new, |kinematics| kinematics.motion_refs.clone()),
      progressive_lods: file.swi_data.as_ref().map_or(0, |swi| swi.windows.len()),
      residue: file.residue.as_ref().map(|residue| OgfResidueReport {
        cause: String::from(residue.cause.as_str()),
        discarded_reference: residue.cause.get_discarded_path().map(String::from),
        position: residue.position,
        size: residue.bytes.len()
          + file
            .kinematics
            .as_ref()
            .map_or(0, |kinematics| kinematics.trailing.len()),
      }),
      shader_id: file.header.shader_id,
      unknown_chunks,
      version: file.header.version,
    }
  }
}
