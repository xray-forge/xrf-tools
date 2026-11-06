use xrf_error::XrfResult;
use xrf_math::{Vector2d, Vector3d};

use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::smoothing_inference::SmoothingInference;
use crate::object::decompile::geometry::two_sided_pairs::TwoSidedPairs;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::mesh::mesh_assembler::MeshAssembler;

/// The editable mesh and surfaces the decompiler builds from `surfaces`, all owned by one bone named `link`.
pub(super) fn assemble(surfaces: &[OgfSurface]) -> XrfResult<AssembledMesh> {
  let mut welded: WeldedMesh = WeldedMesh::weld(surfaces)?;

  welded.faces = TwoSidedPairs::fold(std::mem::take(&mut welded.faces));

  let smoothing: Vec<u32> = SmoothingInference::infer(&welded.faces);

  MeshAssembler::assemble(surfaces, &welded, smoothing, &[String::from("link")])
}

/// A corner at `position` facing `normal`, on bone 0, mapped at the origin of its texture.
pub(super) fn corner(position: [f32; 3], normal: [f32; 3]) -> OgfCorner {
  OgfCorner {
    position: Vector3d::from(position),
    normal: Vector3d::from(normal),
    texture_coordinate: Vector2d::ZERO,
    bone: 0,
  }
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
