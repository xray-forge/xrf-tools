use std::collections::HashSet;

use xrf_chunk::{ChunkReadWrite, ChunkWriter};
use xrf_editable_object::{EditableMesh, EditableObjectFile, EditableSurface};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{DetailModel, DetailVertex};
use xrf_math::Vector3d;

use crate::object::compile::geometry::source::mesh_corner::MeshCorner;
use crate::object::compile::geometry::source::mesh_corners::MeshCorners;
use crate::object::compile::object_compile::ObjectCompile;
use crate::object::detail_parameters::DetailParameters;

/// Builds a detail model (`.dm`) from an editable object as `EDetail::Update` and `EDetail::Export` do.
///
/// The model draws with the one surface's shader and texture, over its first mesh's faces in order whatever surface
/// they name. Corners within `EPS_L` in position and coordinate share a vertex, in first-use order. A face collapsed to
/// fewer than three vertices is dropped, and so is one on a kept face's vertices in any order, a mirror among them.
pub(crate) struct DetailModelBuilder {
  vertices: Vec<DetailVertex>,
  indices: Vec<u16>,
  /// Each kept face's vertices sorted, which every order of the same three shares (`isEqual`).
  kept: HashSet<[u16; 3]>,
  dropped_faces: usize,
}

impl DetailModelBuilder {
  /// `EPS_L`: how close two corners stand, per component of position and coordinate, to be one vertex
  /// (`EDetail::EVertexIn::similar`).
  const SIMILAR: f32 = 1e-3;

  /// The detail model of `object` and `parameters`, written as `EDetail::Export` writes it.
  ///
  /// # Errors
  ///
  /// Returns an error for an object of other than one surface or of no mesh, a corner whose reference's first entry is
  /// not a texture coordinate, more than 65,535 vertices, or a mesh that does not hold together.
  pub(crate) fn build(object: &EditableObjectFile, parameters: &DetailParameters) -> XrfResult<ObjectCompile> {
    let (model, dropped_faces) = Self::build_model(object, parameters)?;
    let mut writer: ChunkWriter = ChunkWriter::new();

    model.write(&mut writer)?;

    Ok(ObjectCompile {
      bytes: writer.flush_raw_into_buffer()?,
      children: 1,
      vertices: model.vertices.len(),
      faces: model.get_triangles_count(),
      dropped_faces,
      tangents: None,
    })
  }

  /// The model [`Self::build`] writes, and how many faces the export dropped.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::build`] does.
  pub(crate) fn build_model(
    object: &EditableObjectFile,
    parameters: &DetailParameters,
  ) -> XrfResult<(DetailModel, usize)> {
    let surfaces: &[EditableSurface] = object.surfaces.as_deref().unwrap_or_default();
    let [surface] = surfaces else {
      return Err(XrfError::new_invalid_error(format!(
        "A detail object holds one surface, not {}",
        surfaces.len()
      )));
    };
    let Some(mesh) = object.meshes.as_deref().and_then(<[EditableMesh]>::first) else {
      return Err(XrfError::new_invalid_error("A detail object holds no mesh"));
    };
    let mut builder: Self = Self {
      vertices: Vec::new(),
      indices: Vec::new(),
      kept: HashSet::new(),
      dropped_faces: 0,
    };

    for corners in MeshCorners::without_normals(mesh).read_faces(DetailVertex::try_from)? {
      builder.add_face(corners)?;
    }

    let model: DetailModel = DetailModel {
      shader: surface.shader.clone(),
      texture: surface.texture.clone(),
      flags: parameters.flags,
      min_scale: parameters.min_scale,
      max_scale: parameters.max_scale,
      vertices: builder.vertices,
      indices: builder.indices,
    };

    Ok((model, builder.dropped_faces))
  }

  fn add_face(&mut self, corners: [DetailVertex; 3]) -> XrfResult {
    let face: [u16; 3] = [
      self.add_vertex(&corners[0])?,
      self.add_vertex(&corners[1])?,
      self.add_vertex(&corners[2])?,
    ];
    let mut sorted: [u16; 3] = face;

    sorted.sort_unstable();

    if sorted[0] == sorted[1] || sorted[1] == sorted[2] || !self.kept.insert(sorted) {
      self.dropped_faces += 1;
    } else {
      self.indices.extend(face);
    }

    Ok(())
  }

  /// `EDetail::_AddVert`: the first vertex similar to `vertex`, or `vertex` added.
  fn add_vertex(&mut self, vertex: &DetailVertex) -> XrfResult<u16> {
    let found: Option<usize> = self
      .vertices
      .iter()
      .position(|existing| Self::is_similar(existing, vertex));
    let index: usize = found.unwrap_or_else(|| {
      self.vertices.push(vertex.clone());
      self.vertices.len() - 1
    });

    u16::try_from(index)
      .map_err(|_| XrfError::new_invalid_error("A detail object holds more vertices than 16-bit indices reach"))
  }

  /// `EVertexIn::similar`: every component of position and coordinate strictly within [`Self::SIMILAR`].
  fn is_similar(first: &DetailVertex, second: &DetailVertex) -> bool {
    let is_near = |a: f32, b: f32| (a - b).abs() < Self::SIMILAR;
    let (first_position, second_position): (&Vector3d, &Vector3d) = (&first.position, &second.position);

    is_near(first.u, second.u)
      && is_near(first.v, second.v)
      && is_near(first_position.x, second_position.x)
      && is_near(first_position.y, second_position.y)
      && is_near(first_position.z, second_position.z)
  }
}

/// A corner's detail vertex: its position and the texture coordinate of its reference's first entry, whatever map it is
/// in.
impl TryFrom<MeshCorner<'_>> for DetailVertex {
  type Error = XrfError;

  fn try_from(corner: MeshCorner<'_>) -> XrfResult<Self> {
    match corner.entries.first().map(|entry| entry.values) {
      Some([u, v, ..]) => Ok(Self {
        position: corner.position,
        u: *u,
        v: *v,
      }),
      _ => Err(XrfError::new_invalid_error(
        "A detail object's face corner names no texture coordinate first",
      )),
    }
  }
}
