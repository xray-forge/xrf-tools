use xrf_chunk::ChunkLayout;
use xrf_editable_object::{
  EditableBone, EditableBoneBindPose, EditableBoneDefinition, EditableBoneJoint, EditableObjectFile,
};
use xrf_error::XrfResult;
use xrf_math::{Cylinder, Obb, Sphere, Vector2d, Vector3d};
use xrf_ogf::OgfVertexLink;
use xrf_skeleton::{SkeletonBoneShape, SkeletonBoneShapeKind, SkeletonJointKind, SkeletonJointLimit};

use crate::object::decompile::geometry::smoothing_inference::SmoothingInference;
use crate::object::decompile::geometry::source::bone_influences::BoneInfluences;
use crate::object::decompile::geometry::source::normal_precision::NormalPrecision;
use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::source::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::two_sided_pairs::TwoSidedPairs;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::mesh::mesh_assembler::MeshAssembler;
use crate::object::decompile::mesh::mesh_kind::MeshKind;

/// The bones of the skeleton the fixtures' corners follow, by index.
pub(crate) const BONES: [&str; 3] = ["link", "lid", "hinge"];

/// A bone under `parent`, its weight map named after it, breakable where `is_breakable`; its physics are what no
/// fixture reads.
pub(crate) fn bone(name: &str, parent: &str, is_breakable: bool) -> EditableBone {
  let limit: SkeletonJointLimit = SkeletonJointLimit {
    min: 0.0,
    max: 0.0,
    spring_factor: 1.0,
    damping_factor: 1.0,
  };

  EditableBone {
    version: EditableBone::CURRENT_VERSION,
    definition: EditableBoneDefinition {
      name: String::from(name),
      parent: String::from(parent),
      weight_map: String::from(name),
    },
    bind_pose: EditableBoneBindPose {
      offset: Vector3d::ZERO,
      rotation: Vector3d::ZERO,
      length: 0.5,
    },
    data_name: None,
    game_material: String::from("objects\\tin_can"),
    shape: SkeletonBoneShape {
      kind: SkeletonBoneShapeKind::default(),
      flags: 0,
      obb: Obb {
        rotation: [Vector3d::X, Vector3d::Y, Vector3d::Z],
        center: Vector3d::ZERO,
        half_extents: Vector3d::ZERO,
      },
      sphere: Sphere {
        center: Vector3d::ZERO,
        radius: 0.0,
      },
      cylinder: Cylinder {
        center: Vector3d::ZERO,
        direction: Vector3d::Y,
        height: 0.0,
        radius: 0.0,
      },
    },
    joint_flags: Some(u32::from(is_breakable)),
    joint: EditableBoneJoint {
      kind: SkeletonJointKind::default(),
      limits: [limit.clone(), limit.clone(), limit],
      spring_factor: 1.0,
      damping_factor: 1.0,
    },
    joint_break: None,
    friction: None,
    mass: None,
    layout: ChunkLayout::default(),
  }
}

/// [`BONES`] in a chain from `link`, none breakable.
pub(crate) fn chain() -> Vec<EditableBone> {
  vec![
    bone(BONES[0], "", false),
    bone(BONES[1], BONES[0], false),
    bone(BONES[2], BONES[1], false),
  ]
}

/// A dynamic object of `assembled`'s mesh and surfaces on a skeleton of `bones`.
pub(crate) fn object(assembled: AssembledMesh, bones: Vec<EditableBone>) -> EditableObjectFile {
  EditableObjectFile {
    flags: EditableObjectFile::DYNAMIC_FLAG,
    flags_size: EditableObjectFile::FLAGS_SIZE,
    class_script: None,
    lods: None,
    meshes: Some(vec![assembled.mesh]),
    surfaces: Some(assembled.surfaces),
    bones: Some(bones),
    embedded_motions: None,
    motion_references: None,
    bone_parts: None,
    actor_transform: None,
    description: None,
    layout: ChunkLayout::default(),
  }
}

/// The editable mesh and surfaces the decompiler builds from `surfaces`, on a skeleton of [`BONES`].
pub(crate) fn assemble(surfaces: &[OgfSurface]) -> XrfResult<AssembledMesh> {
  assemble_as(
    surfaces,
    MeshKind::Skinned {
      bone_names: &BONES.map(String::from),
    },
  )
}

/// The editable mesh and surfaces the decompiler builds from `surfaces` of [`unweighted`] corners, for a static object.
pub(crate) fn assemble_static(surfaces: &[OgfSurface]) -> XrfResult<AssembledMesh> {
  assemble_as(surfaces, MeshKind::Static)
}

fn assemble_as(surfaces: &[OgfSurface], kind: MeshKind<'_>) -> XrfResult<AssembledMesh> {
  let mut welded: WeldedMesh = WeldedMesh::weld(surfaces)?;

  welded.faces = TwoSidedPairs::fold(std::mem::take(&mut welded.faces));

  let smoothing: Vec<u32> = SmoothingInference::infer(&welded.faces);

  MeshAssembler::assemble(surfaces, &welded, smoothing, kind)
}

/// A static object of `assembled`'s mesh and surfaces: no bones, neither dynamic nor progressive.
pub(crate) fn static_object(assembled: AssembledMesh) -> EditableObjectFile {
  EditableObjectFile {
    flags: 0,
    bones: None,
    ..object(assembled, Vec::new())
  }
}

/// The same corner following no bone, as a static vertex's does.
pub(crate) fn unweighted(corner: OgfCorner) -> OgfCorner {
  OgfCorner {
    influences: BoneInfluences::NONE,
    ..corner
  }
}

/// A corner at `position` facing `normal`, following bone 0 alone, mapped at the origin of its texture.
pub(crate) fn corner(position: [f32; 3], normal: [f32; 3]) -> OgfCorner {
  OgfCorner {
    position: Vector3d::from(position),
    normal: Vector3d::from(normal),
    normal_precision: NormalPrecision::Float,
    texture_coordinate: Vector2d::ZERO,
    influences: BoneInfluences::of_bone(0),
  }
}

/// The same corner following the bones `links` stand for.
pub(crate) fn with_links(corner: OgfCorner, links: &[(u16, f32)]) -> OgfCorner {
  OgfCorner {
    influences: BoneInfluences::of_links(
      &links
        .iter()
        .map(|(bone, weight)| link(*bone, *weight))
        .collect::<Vec<_>>(),
    ),
    ..corner
  }
}

pub(crate) fn link(bone: u16, weight: f32) -> OgfVertexLink {
  OgfVertexLink { bone, weight }
}

/// The same corner mapped at `uv`.
pub(crate) fn with_uv(corner: OgfCorner, uv: [f32; 2]) -> OgfCorner {
  OgfCorner {
    texture_coordinate: Vector2d::from(uv),
    ..corner
  }
}

/// A surface of `triangles` drawn with one texture.
pub(crate) fn surface(triangles: Vec<[OgfCorner; 3]>) -> OgfSurface {
  OgfSurface {
    texture: String::from("wood\\wood_plank6"),
    shader: String::from("models\\model"),
    triangles,
  }
}

pub(crate) const UP: [f32; 3] = [0.0, 1.0, 0.0];
pub(crate) const DOWN: [f32; 3] = [0.0, -1.0, 0.0];
pub(crate) const OUT: [f32; 3] = [0.0, 0.0, 1.0];
