use xrf_chunk::ChunkLayout;
use xrf_editable_object::{EditableFace, EditableFaceCorner, EditableMesh, EditableSurfaceFaces};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Aabb;

use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::source::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::mesh::mesh_kind::MeshKind;
use crate::object::decompile::mesh::surface_layout::SurfaceLayout;
use crate::object::decompile::mesh::vertex_maps::VertexMaps;

/// Builds the editable mesh and its surfaces from welded faces, laid out as the SDK lays out an object of `kind`.
///
/// One mesh, whose every corner refers to its UV and, in a skinned mesh, to its point's weight for each bone the point
/// follows: a point UV map, a weight map named after each bone, and a polygon-mapped UV map for the corners whose UV is
/// not their point's, which is how a seam gives one point two UVs.
pub(crate) struct MeshAssembler;

impl MeshAssembler {
  /// The Actor Editor's compiler shader and game material for a new surface, which the OGF does not keep.
  pub(crate) const DEFAULT_SURFACE_SETTING: &'static str = "default";

  /// Assembles the mesh.
  ///
  /// # Errors
  ///
  /// Returns an error when the mesh holds no point, a point's bone is past those `kind` names, or an index outgrows
  /// the `u32` an `.object` stores it as.
  pub(crate) fn assemble(
    surfaces: &[OgfSurface],
    welded: &WeldedMesh,
    smoothing: Vec<u32>,
    kind: MeshKind<'_>,
  ) -> XrfResult<AssembledMesh> {
    let layout: SurfaceLayout = SurfaceLayout::of(surfaces, &welded.faces);
    let mut maps: VertexMaps = VertexMaps::new(welded, kind)?;
    let mut faces: Vec<EditableFace> = Vec::with_capacity(welded.faces.len());
    let mut surface_faces: Vec<Vec<u32>> = vec![Vec::new(); layout.surfaces.len()];

    for (face_index, face) in welded.faces.iter().enumerate() {
      let face_number: u32 = Self::narrow_index(face_index, "faces")?;
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
        name: String::from(kind.get_mesh_name()),
        bounds,
        flags: EditableMesh::VISIBLE_FLAG,
        options: Some([0, 0]),
        points: welded.points.clone(),
        has_point_faces: false,
        faces,
        smoothing_groups: Some(smoothing),
        // The normals the OGF stores, which the smoothing only approximates where another exporter's rule made them:
        // the compile takes them over the SDK's, and `blender-xray` imports them; the SDK reads only the smoothing. A
        // detail model stores none, so its object leaves them to the smoothing.
        corner_normals: welded
          .faces
          .iter()
          .all(|face| face.corners.iter().all(OgfCorner::has_normal))
          .then(|| {
            welded
              .faces
              .iter()
              .map(|face| face.corners.each_ref().map(|corner| corner.normal))
              .collect()
          }),
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
  pub(crate) fn narrow_index(value: usize, what: &str) -> XrfResult<u32> {
    u32::try_from(value)
      .map_err(|_| XrfError::new_invalid_error(format!("A decompiled mesh has more {what} than a u32 indexes")))
  }
}
