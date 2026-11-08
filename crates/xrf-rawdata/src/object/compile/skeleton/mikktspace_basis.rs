use std::collections::HashMap;

use bevy_mikktspace::{Geometry, TangentSpace, generate_tangents};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::object::compile::geometry::export_vertex::ExportVertex;
use crate::object::compile::geometry::tangent_basis::TangentBasis;

/// Gives a split's vertices MikkTSpace tangents and binormals, the basis Blender bakes and `blender-xray` exports
/// against, in place of the Actor Editor's (see [`TangentBasis`]).
///
/// MikkTSpace gives each face corner a unit tangent along the texture's `u`, squared to the normal, and a sign from the
/// texture's orientation across the face; the binormal is `sign · (normal × tangent)`, along `v` as the export's runs.
/// On the OGF's texture coordinates, whose `v` runs the other way from Blender's, it is the negated Blender bitangent
/// `blender-xray` stores. A vertex is split where its
/// corners' bases differ, the vertices are kept in the order the faces first reach them, and texture coordinates are
/// moved by whole tiles as the export moves them. A corner MikkTSpace gives no basis, a degenerate face's with no
/// neighbour to borrow one from, takes one built from its normal.
pub(crate) struct MikkTSpaceBasis;

/// A split as [`Geometry`] reads it, and the basis it gives each face corner.
struct SplitGeometry<'a> {
  vertices: &'a [ExportVertex],
  faces: &'a [[u32; 3]],
  bases: Vec<[Option<(Vector3d, Vector3d)>; 3]>,
}

impl SplitGeometry<'_> {
  fn vertex(&self, face: usize, corner: usize) -> &ExportVertex {
    &self.vertices[self.faces[face][corner] as usize]
  }
}

impl Geometry for SplitGeometry<'_> {
  fn num_faces(&self) -> usize {
    self.faces.len()
  }

  fn num_vertices_of_face(&self, _face: usize) -> usize {
    3
  }

  fn position(&self, face: usize, vert: usize) -> [f32; 3] {
    self.vertex(face, vert).position.to_array()
  }

  fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
    self.vertex(face, vert).normal.to_array()
  }

  fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
    let coordinate = &self.vertex(face, vert).texture_coordinate;

    [coordinate.x, coordinate.y]
  }

  fn set_tangent(&mut self, tangent_space: Option<TangentSpace>, face: usize, vert: usize) {
    let normal: Vector3d = self.vertex(face, vert).normal;

    self.bases[face][vert] = tangent_space.map(|space| {
      let [x, y, z, sign] = space.tangent_encoded();
      let tangent: Vector3d = Vector3d::new(x, y, z);

      (tangent, &normal.cross(&tangent) * sign)
    });
  }
}

impl MikkTSpaceBasis {
  /// `vertices` and `faces` with MikkTSpace tangents and binormals, the vertices split where their corners' bases
  /// differ, ordered by first use, and texture coordinates moved by whole tiles.
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a vertex the split lacks, or the split outgrows 32-bit indices.
  pub(crate) fn generate(
    vertices: Vec<ExportVertex>,
    faces: &[[u32; 3]],
  ) -> XrfResult<(Vec<ExportVertex>, Vec<[u32; 3]>)> {
    if faces.iter().flatten().any(|index| *index as usize >= vertices.len()) {
      return Err(XrfError::new_invalid_error(
        "A skeleton split face names a vertex the split lacks",
      ));
    }

    let mut geometry: SplitGeometry = SplitGeometry {
      vertices: &vertices,
      faces,
      bases: vec![[None; 3]; faces.len()],
    };

    generate_tangents(&mut geometry)
      .map_err(|_| XrfError::new_invalid_error("MikkTSpace gives a skeleton split no tangent space"))?;

    let bases: Vec<[Option<(Vector3d, Vector3d)>; 3]> = geometry.bases;
    let mut ordered: Vec<ExportVertex> = Vec::with_capacity(vertices.len());
    let mut index_of: HashMap<(u32, [u32; 6]), u32> = HashMap::with_capacity(vertices.len());
    let mut renumbered: Vec<[u32; 3]> = Vec::with_capacity(faces.len());

    for (face, face_bases) in faces.iter().zip(bases) {
      let mut indices: [u32; 3] = [0; 3];

      for ((slot, source), basis) in indices.iter_mut().zip(face).zip(face_bases) {
        let vertex: &ExportVertex = &vertices[*source as usize];
        let (tangent, binormal) = basis.unwrap_or_else(|| TangentBasis::get_basis_from_normal(&vertex.normal));
        let bits: [u32; 6] = [tangent.x, tangent.y, tangent.z, binormal.x, binormal.y, binormal.z].map(f32::to_bits);

        *slot = match index_of.get(&(*source, bits)) {
          Some(index) => *index,
          None => {
            let index: u32 = u32::try_from(ordered.len()).map_err(|_| {
              XrfError::new_invalid_error("A skeleton split outgrows 32-bit indices once its tangents split it")
            })?;

            ordered.push(ExportVertex {
              tangent,
              binormal,
              ..vertex.clone()
            });
            index_of.insert((*source, bits), index);

            index
          }
        };
      }

      renumbered.push(indices);
    }

    TangentBasis::shift_texture_coordinates(&mut ordered);

    Ok((ordered, renumbered))
  }
}
