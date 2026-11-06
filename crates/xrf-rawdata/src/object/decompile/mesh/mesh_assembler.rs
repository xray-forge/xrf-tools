use xrf_chunk::ChunkLayout;
use xrf_editable_object::{EditableFace, EditableFaceCorner, EditableMesh, EditableSurfaceFaces};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Aabb;

use crate::object::decompile::geometry::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::mesh::surface_layout::SurfaceLayout;
use crate::object::decompile::mesh::vertex_maps::VertexMaps;

/// Builds the editable mesh and its surfaces from welded faces, laid out as the SDK lays out a skinned object.
///
/// One mesh, `skin`, whose every corner refers to two vertex map entries, its UV and its bone weight: a point UV map
/// `texture`, a weight map named after each bone, and a polygon-mapped `texture` for the corners whose UV is not their
/// point's, which is how a seam gives one point two UVs. A rigid model weighs each point fully to its one bone.
pub(crate) struct MeshAssembler;

impl MeshAssembler {
  /// The name the SDK gives a skinned object's mesh.
  pub(crate) const MESH_NAME: &'static str = "skin";

  /// The name of the mesh's UV maps.
  pub(crate) const UV_MAP_NAME: &'static str = "texture";

  /// The Actor Editor's compiler shader and game material for a new surface, which the OGF does not keep.
  pub(crate) const DEFAULT_SURFACE_SETTING: &'static str = "default";

  /// Assembles the mesh.
  ///
  /// # Errors
  ///
  /// Returns an error when the mesh holds no point, a point's bone is past `bone_names`, or an index outgrows the
  /// `u32` an `.object` stores it as.
  pub(crate) fn assemble(
    surfaces: &[OgfSurface],
    welded: &WeldedMesh,
    smoothing: Vec<u32>,
    bone_names: &[String],
  ) -> XrfResult<AssembledMesh> {
    let layout: SurfaceLayout = SurfaceLayout::of(surfaces, &welded.faces);
    let mut maps: VertexMaps = VertexMaps::new(welded, bone_names)?;
    let mut faces: Vec<EditableFace> = Vec::with_capacity(welded.faces.len());
    let mut surface_faces: Vec<Vec<u32>> = vec![Vec::new(); layout.surfaces.len()];

    for (face_index, face) in welded.faces.iter().enumerate() {
      let face_number: u32 = Self::to_index(face_index, "faces")?;
      let mut corners: [EditableFaceCorner; 3] = [EditableFaceCorner {
        point: 0,
        vmap_reference: 0,
      }; 3];

      for (corner, (point, ogf)) in corners.iter_mut().zip(face.points.iter().zip(&face.corners)) {
        *corner = EditableFaceCorner {
          point: *point,
          vmap_reference: maps.refer(*point, face_number, ogf.texture_coordinate)?,
        };
      }

      faces.push(EditableFace { corners });
      surface_faces[layout.get_surface(face.surface, face.is_two_sided)].push(face_number);
    }

    let bounds: Aabb = Aabb::from_points(&welded.points)
      .ok_or_else(|| XrfError::new_invalid_error("A decompiled mesh has no points"))?;

    Ok(AssembledMesh {
      mesh: EditableMesh {
        name: String::from(Self::MESH_NAME),
        bounds,
        flags: EditableMesh::VISIBLE_FLAG,
        options: Some([0, 0]),
        points: welded.points.clone(),
        has_point_faces: false,
        faces,
        smoothing_groups: Some(smoothing),
        vmap_references: std::mem::take(&mut maps.references),
        surface_faces: layout
          .surfaces
          .iter()
          .zip(surface_faces)
          .map(|(surface, faces)| EditableSurfaceFaces {
            surface: surface.name.clone(),
            faces,
          })
          .collect(),
        vmaps: Some(maps.into_vmaps()),
        layout: ChunkLayout::default(),
      },
      surfaces: layout.surfaces,
    })
  }

  /// `value` as the `u32` an `.object` stores counts and indices as.
  ///
  /// # Errors
  ///
  /// Returns an error when it does not fit, naming `what` it counts.
  pub(crate) fn to_index(value: usize, what: &str) -> XrfResult<u32> {
    u32::try_from(value)
      .map_err(|_| XrfError::new_invalid_error(format!("A decompiled mesh has more {what} than a u32 indexes")))
  }
}
