use byteorder::ByteOrder;
use xrf_chunk::{ChunkReadWrite, ChunkWriter};
use xrf_editable_object::{EditableBone, EditableObjectFile, EditableTimeWidth};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{
  OgfBone, OgfBoneIkData, OgfBonesChunk, OgfChildrenChunk, OgfDescriptionChunk, OgfDescriptionTimeWidth, OgfGeometry,
  OgfHeaderChunk, OgfIkDataChunk, OgfJointIkData, OgfKinematicsChunk, OgfLodsChunk, OgfModelType, OgfTextureChunk,
  OgfUserDataChunk, OgfVertex, OgfVertexLink, OgfVertexLinks,
};
use xrf_skeleton::{SkeletonMotionParametersChunk, SkeletonMotionsChunk};

use crate::object::compile::motion::motion_export::MotionExport;
use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::skeleton::bone_boxes::BoneBoxes;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;
use crate::object::compile::skeleton::skeleton_vertex::{SkeletonVertex, SkeletonVertexBone};
use crate::object::joint_limit_sense::JointLimitSense;

/// Writes a skeleton object's OGF as `CExportSkeleton::ExportGeometry` and `ExportMotions` do: the header, the
/// description, a child per split, the bones' names and boxes, their physics, the class script, the LODs, then the
/// object's own motions and how they play, or the motion files it references.
pub(crate) struct SkeletonOgfWriter;

impl SkeletonOgfWriter {
  /// `xrOGF_FormatVersion`.
  const FORMAT_VERSION: u8 = 4;

  /// `OGF_IKDATA_VERSION`: a record carrying the joint's friction.
  const IK_DATA_VERSION: u32 = 1;

  /// `CBone::ResetData`'s mass, for a bone storing none.
  const DEFAULT_MASS: f32 = 10.0;

  /// The OGF's bytes.
  ///
  /// # Errors
  ///
  /// Returns an error where the object's own motions do not export (see [`MotionExport::of`]), and where a chunk does
  /// not write, such as a description time past the 32-bit times it was authored with.
  pub(crate) fn write<T: ByteOrder>(
    object: &EditableObjectFile,
    geometry: &SkeletonGeometry,
    source: &ObjectCompileSource,
  ) -> XrfResult<Vec<u8>> {
    let bones: &[EditableBone] = object.bones.as_deref().unwrap_or_default();
    let motion_references: &[String] = object.motion_references.as_deref().unwrap_or_default();
    let motions: Option<MotionExport> = MotionExport::of::<T>(object)?;
    let mut file: ChunkWriter = ChunkWriter::new();

    Self::write_chunk::<T, _>(
      &mut file,
      OgfHeaderChunk::CHUNK_ID,
      &Self::header(
        if motion_references.is_empty() && motions.is_none() {
          OgfModelType::SkeletonRigid
        } else {
          OgfModelType::SkeletonAnim
        },
        &Self::get_bounds(geometry.splits.iter().flat_map(|split| &split.vertices)),
      ),
    )?;
    Self::write_chunk::<T, _>(
      &mut file,
      OgfDescriptionChunk::CHUNK_ID,
      &Self::description(object, source),
    )?;

    let children: Vec<Vec<u8>> = geometry
      .splits
      .iter()
      .map(Self::write_child::<T>)
      .collect::<XrfResult<_>>()?;

    OgfChildrenChunk::write_nested::<T>(&mut file, &children)?;

    Self::write_chunk::<T, _>(
      &mut file,
      OgfBonesChunk::CHUNK_ID,
      &OgfBonesChunk {
        bones: bones
          .iter()
          .zip(BoneBoxes::of(bones, &geometry.splits))
          .map(|(bone, obb)| OgfBone {
            name: bone.definition.name.clone(),
            parent: bone.definition.parent.clone(),
            obb,
          })
          .collect(),
      },
    )?;

    let mut ik_data: ChunkWriter = ChunkWriter::new();

    OgfIkDataChunk {
      bones: bones.iter().map(Self::ik_data).collect(),
    }
    .write::<T>(&mut ik_data)?;
    ik_data.flush_chunk_into::<T>(&mut file, OgfIkDataChunk::CHUNK_ID)?;

    // The class script is written terminated (`w_stringZ`), the LODs as a line (`w_string`).
    if let Some(script) = object.class_script.as_deref().filter(|script| !script.is_empty()) {
      Self::write_chunk::<T, _>(
        &mut file,
        OgfUserDataChunk::CHUNK_ID,
        &OgfUserDataChunk {
          user_data: format!("{script}\0"),
        },
      )?;
    }

    if let Some(lods) = object.lods.as_deref().filter(|lods| !lods.is_empty()) {
      Self::write_chunk::<T, _>(
        &mut file,
        OgfLodsChunk::CHUNK_ID,
        &OgfLodsChunk {
          lods: format!("{lods}\r\n"),
        },
      )?;
    }

    // An object referencing motion files plays those and writes none of its own (`ExportMotionKeys`).
    if let Some(motions) = &motions {
      Self::write_chunk::<T, _>(&mut file, SkeletonMotionsChunk::CHUNK_ID, &motions.motions)?;
      Self::write_chunk::<T, _>(&mut file, SkeletonMotionParametersChunk::CHUNK_ID, &motions.parameters)?;
    } else if !motion_references.is_empty() {
      let mut kinematics: ChunkWriter = ChunkWriter::new();

      OgfKinematicsChunk {
        source_chunk_id: OgfKinematicsChunk::CHUNK_ID,
        motion_refs: motion_references.to_vec(),
        trailing: Vec::new(),
      }
      .write::<T>(&mut kinematics)?;
      kinematics.flush_chunk_into::<T>(&mut file, OgfKinematicsChunk::CHUNK_ID)?;
    }

    file.flush_raw_into_buffer()
  }

  /// A split's visual, `CExportSkeleton::SSplit::Save`: a static skinned child (no sliding windows yet), its texture,
  /// vertices and indices.
  fn write_child<T: ByteOrder>(split: &SkeletonSplit) -> XrfResult<Vec<u8>> {
    let mut child: ChunkWriter = ChunkWriter::new();

    Self::write_chunk::<T, _>(
      &mut child,
      OgfHeaderChunk::CHUNK_ID,
      &Self::header(OgfModelType::SkeletonGeomdefSt, &Self::get_bounds(&split.vertices)),
    )?;
    Self::write_chunk::<T, _>(
      &mut child,
      OgfTextureChunk::CHUNK_ID,
      &OgfTextureChunk {
        texture_name: split.texture.clone(),
        shader_name: split.shader.clone(),
        trailing: Vec::new(),
      },
    )?;

    let vertices: Vec<OgfVertex> = split
      .vertices
      .iter()
      .map(|vertex| Self::to_ogf_vertex(vertex, split.link_count))
      .collect();
    let indices: Vec<u16> = split.faces.iter().flatten().copied().collect();

    OgfGeometry::write_skinned::<T>(&mut child, &vertices, &indices)?;

    child.flush_raw_into_buffer()
  }

  /// A vertex laid out in its split's `link_count` as `Save` lays it: bones strongest first, a two-link vertex
  /// storing the second bone's share of the two, padding repeating the first bone weightless.
  fn to_ogf_vertex(vertex: &SkeletonVertex, link_count: u8) -> OgfVertex {
    let mut bones: Vec<SkeletonVertexBone> = vertex.bones.clone();

    // Strongest first, equals as they came: the export's `sort_by_weight` over a handful of bones.
    bones.sort_by(|first, second| second.weight.total_cmp(&first.weight));

    let first: SkeletonVertexBone = bones[0];
    let link = |bone: u16, weight: f32| OgfVertexLink { bone, weight };
    let links: Vec<OgfVertexLink> = match link_count {
      1 => vec![link(first.bone, 1.0)],
      2 => {
        let (second, stored): (u16, f32) = match bones.get(1) {
          Some(second) => (second.bone, second.weight / (first.weight + second.weight)),
          None => (first.bone, 1.0),
        };

        vec![link(first.bone, 1.0 - stored), link(second, stored)]
      }
      count => {
        let count: usize = usize::from(count);
        let mut links: Vec<OgfVertexLink> = (0..count)
          .map(|index| match bones.get(index) {
            Some(bone) => link(bone.bone, bone.weight),
            None => link(first.bone, 0.0),
          })
          .collect();
        let stored: f32 = links[..count - 1].iter().map(|link| link.weight).sum();

        links[count - 1].weight = 1.0 - stored;
        links
      }
    };

    OgfVertex {
      position: vertex.position,
      normal: vertex.normal,
      tangent: vertex.tangent,
      binormal: vertex.binormal,
      texture_u: vertex.texture_coordinate.x,
      texture_v: vertex.texture_coordinate.y,
      links: OgfVertexLinks::from_slice(&links),
    }
  }

  /// A bone's physics record, `CBone::ExportOGF`, its joint limits stored for the physics engine.
  fn ik_data(bone: &EditableBone) -> OgfBoneIkData {
    OgfBoneIkData {
      version: Self::IK_DATA_VERSION,
      game_material: bone.game_material.clone(),
      shape: bone.shape.clone(),
      joint: OgfJointIkData {
        kind: bone.joint.kind,
        limits: bone.joint.limits.clone().map(JointLimitSense::flip),
        spring_factor: bone.joint.spring_factor,
        damping_factor: bone.joint.damping_factor,
        ik_flags: bone.joint_flags.unwrap_or(0),
        break_force: bone.joint_break.as_ref().map_or(0.0, |it| it.force),
        break_torque: bone.joint_break.as_ref().map_or(0.0, |it| it.torque),
        friction: Some(bone.friction.unwrap_or(0.0)),
      },
      bind_rotation: bone.bind_pose.rotation,
      bind_position: bone.bind_pose.offset,
      mass: bone.mass.as_ref().map_or(Self::DEFAULT_MASS, |it| it.mass),
      center_of_mass: bone.mass.as_ref().map_or(Vector3d::ZERO, |it| it.center),
    }
  }

  /// The description `CEditableObject::PrepareOGFDesc` fills: the source, what built it when, and who made and last
  /// changed the object when.
  fn description(object: &EditableObjectFile, source: &ObjectCompileSource) -> OgfDescriptionChunk {
    let authored = object.description.as_ref();

    OgfDescriptionChunk {
      source_file: source.source_file.clone(),
      convertor: source.convertor.clone(),
      built_at: source.built_at,
      creator: authored.map(|it| it.creator.clone()).unwrap_or_default(),
      created_at: authored.map_or(0, |it| it.created_at),
      editor: authored.map(|it| it.modifier.clone()).unwrap_or_default(),
      edited_at: authored.map_or(0, |it| it.modified_at),
      time_width: match authored.map(|it| it.time_width) {
        Some(EditableTimeWidth::Seconds64) => OgfDescriptionTimeWidth::Wide,
        _ => OgfDescriptionTimeWidth::Narrow,
      },
    }
  }

  fn header(model_type: OgfModelType, bounds: &Aabb) -> OgfHeaderChunk {
    let center: Vector3d = bounds.min.midpoint(&bounds.max);

    OgfHeaderChunk {
      version: Self::FORMAT_VERSION,
      model_type: model_type.to_raw(),
      shader_id: 0,
      bounding_box: *bounds,
      // `Fbox::getsphere`: the centre, out to the box's upper corner.
      bounding_sphere: Sphere {
        center,
        radius: center.distance(&bounds.max),
      },
    }
  }

  fn get_bounds<'a>(vertices: impl IntoIterator<Item = &'a SkeletonVertex>) -> Aabb {
    let mut vertices = vertices.into_iter();
    let Some(first) = vertices.next() else {
      return Aabb {
        min: Vector3d::ZERO,
        max: Vector3d::ZERO,
      };
    };

    vertices.fold(
      Aabb {
        min: first.position,
        max: first.position,
      },
      |bounds, vertex| Aabb {
        min: bounds.min.min(&vertex.position),
        max: bounds.max.max(&vertex.position),
      },
    )
  }

  fn write_chunk<T: ByteOrder, C: ChunkReadWrite>(writer: &mut ChunkWriter, id: u32, chunk: &C) -> XrfResult {
    let mut contents: ChunkWriter = ChunkWriter::new();

    chunk.write::<T>(&mut contents)?;
    contents.flush_chunk_into::<T>(writer, id)?;

    Ok(())
  }
}
