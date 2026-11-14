use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{Mesh, MeshVertex, TangentBasis};
use xrf_math::Vector3d;

use crate::geometry::geometry_basis::GeometryBasis;
use crate::geometry::geometry_visual::GeometryVisual;
use crate::geometry::geometry_visual_vertex::GeometryVisualVertex;

impl GeometryVisual {
  /// `OGF::CalculateTB` (`xrLC/OGF_CalculateTB.cpp:11`), as a multi-use model's visuals take their basis: the vertices
  /// ordered by first use, the mender run over the visual alone with the arguments both exports give it, each vertex
  /// it splits a copy of its source, and the vertices ordered by first use again.
  ///
  /// # Errors
  ///
  /// Returns an error when the mended visual outgrows 16-bit indices.
  pub(crate) fn mend_basis(&mut self) -> XrfResult {
    let ordered: Mesh<u32> = TangentBasis::order_by_first_use(Mesh {
      vertices: self
        .vertices
        .iter()
        .map(|vertex| MeshVertex {
          position: vertex.position,
          normal: vertex.normal,
          tangent: Vector3d::ZERO,
          binormal: Vector3d::ZERO,
          texture_coordinate: vertex.texture_coordinate,
          bones: Vec::new(),
        })
        .collect(),
      faces: self.faces.iter().map(|face| face.map(u32::from)).collect(),
    })?;
    let mended: Mesh<u32> = TangentBasis::order_by_first_use(TangentBasis::mend_in_place(ordered))?;

    self.vertices = mended
      .vertices
      .iter()
      .map(|vertex| GeometryVisualVertex {
        position: vertex.position,
        normal: vertex.normal,
        basis: GeometryBasis::of(&vertex.tangent, &vertex.binormal),
        texture_coordinate: vertex.texture_coordinate,
        lightmap_coordinate: None,
      })
      .collect();
    self.faces = mended
      .faces
      .iter()
      .map(|face| {
        let mut indices: [u16; 3] = [0; 3];

        for (slot, index) in indices.iter_mut().zip(face) {
          *slot = u16::try_from(*index)
            .map_err(|_| XrfError::new_invalid_error("A multi-use visual outgrows 16-bit indices once mended"))?;
        }

        Ok(indices)
      })
      .collect::<XrfResult<_>>()?;

    Ok(())
  }
}
