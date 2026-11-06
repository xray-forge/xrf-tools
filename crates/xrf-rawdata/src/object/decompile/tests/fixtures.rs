use xrf_error::XrfResult;
use xrf_math::{Vector2d, Vector3d};
use xrf_ogf::OgfVertexLink;

use crate::object::decompile::geometry::bone_influences::BoneInfluences;
use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::smoothing_inference::SmoothingInference;
use crate::object::decompile::geometry::two_sided_pairs::TwoSidedPairs;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::mesh::mesh_assembler::MeshAssembler;

/// The bones of the skeleton the fixtures' corners follow, by index.
pub(super) const BONES: [&str; 3] = ["link", "lid", "hinge"];

/// The editable mesh and surfaces the decompiler builds from `surfaces`, on a skeleton of [`BONES`].
pub(super) fn assemble(surfaces: &[OgfSurface]) -> XrfResult<AssembledMesh> {
  let mut welded: WeldedMesh = WeldedMesh::weld(surfaces)?;

  welded.faces = TwoSidedPairs::fold(std::mem::take(&mut welded.faces));

  let smoothing: Vec<u32> = SmoothingInference::infer(&welded.faces);

  MeshAssembler::assemble(surfaces, &welded, smoothing, &BONES.map(String::from))
}

/// A corner at `position` facing `normal`, following bone 0 alone, mapped at the origin of its texture.
pub(super) fn corner(position: [f32; 3], normal: [f32; 3]) -> OgfCorner {
  OgfCorner {
    position: Vector3d::from(position),
    normal: Vector3d::from(normal),
    texture_coordinate: Vector2d::ZERO,
    influences: BoneInfluences::of_bone(0),
  }
}

/// The same corner following the bones `links` stand for.
pub(super) fn with_links(corner: OgfCorner, links: &[(u16, f32)]) -> OgfCorner {
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

pub(super) fn link(bone: u16, weight: f32) -> OgfVertexLink {
  OgfVertexLink { bone, weight }
}

/// The same corner mapped at `uv`.
pub(super) fn with_uv(corner: OgfCorner, uv: [f32; 2]) -> OgfCorner {
  OgfCorner {
    texture_coordinate: Vector2d::from(uv),
    ..corner
  }
}

/// A surface of `triangles` drawn with one texture.
pub(super) fn surface(triangles: Vec<[OgfCorner; 3]>) -> OgfSurface {
  OgfSurface {
    texture: String::from("wood\\wood_plank6"),
    shader: String::from("models\\model"),
    triangles,
  }
}

pub(super) const UP: [f32; 3] = [0.0, 1.0, 0.0];
pub(super) const DOWN: [f32; 3] = [0.0, -1.0, 0.0];
pub(super) const OUT: [f32; 3] = [0.0, 0.0, 1.0];
