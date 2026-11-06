use xrf_chunk::ChunkLayout;
use xrf_editable_object::{
  EditableBone, EditableBoneBindPose, EditableBoneBreak, EditableBoneDefinition, EditableBoneJoint, EditableBoneMass,
};
use xrf_error::{XrfError, XrfResult};
use xrf_ogf::{OgfBone, OgfBoneIkData, OgfFile};

use crate::object::joint_limit_sense::JointLimitSense;

/// Rebuilds an editable object's bones from an OGF's bones chunk and its IK data, which `CBone::ExportOGF` wrote from
/// them.
pub(crate) struct BoneConverter;

impl BoneConverter {
  /// The length `CBone` starts with, which no OGF stores.
  pub(crate) const DEFAULT_LENGTH: f32 = 0.5;

  /// Every bone, in the OGF's order.
  ///
  /// # Errors
  ///
  /// Returns an error when the OGF has no bones, or its IK data does not hold one record per bone.
  pub(crate) fn convert(file: &OgfFile) -> XrfResult<Vec<EditableBone>> {
    let bones: &[OgfBone] = file
      .bones
      .as_ref()
      .map(|chunk| chunk.bones.as_slice())
      .ok_or_else(|| XrfError::new_invalid_error("A skeleton OGF has no bones chunk"))?;
    let records: &[OgfBoneIkData] = file.ik_data.as_ref().map_or(&[], |chunk| chunk.bones.as_slice());

    if records.len() != bones.len() {
      return Err(XrfError::new_invalid_error(format!(
        "A skeleton OGF has {} bones and IK data for {}",
        bones.len(),
        records.len()
      )));
    }

    Ok(
      bones
        .iter()
        .zip(records)
        .map(|(bone, record)| Self::convert_bone(bone, record))
        .collect(),
    )
  }

  fn convert_bone(bone: &OgfBone, record: &OgfBoneIkData) -> EditableBone {
    EditableBone {
      version: EditableBone::CURRENT_VERSION,
      definition: EditableBoneDefinition {
        name: bone.name.clone(),
        parent: bone.parent.clone(),
        weight_map: bone.name.clone(),
      },
      bind_pose: EditableBoneBindPose {
        offset: record.bind_position,
        rotation: record.bind_rotation,
        length: Self::DEFAULT_LENGTH,
      },
      data_name: Some(bone.name.clone()),
      game_material: record.game_material.clone(),
      shape: record.shape.clone(),
      joint_flags: Some(record.joint.ik_flags),
      joint: EditableBoneJoint {
        kind: record.joint.kind,
        limits: record.joint.limits.clone().map(JointLimitSense::flip),
        spring_factor: record.joint.spring_factor,
        damping_factor: record.joint.damping_factor,
      },
      joint_break: Some(EditableBoneBreak {
        force: record.joint.break_force,
        torque: record.joint.break_torque,
      }),
      friction: record.joint.friction,
      mass: Some(EditableBoneMass {
        mass: record.mass,
        center: record.center_of_mass,
      }),
      layout: ChunkLayout::default(),
    }
  }
}
