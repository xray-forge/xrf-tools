use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{Mesh, MeshVertex, TangentBasis};
use xrf_math::{Vector2d, Vector3d};

use crate::geometry::surface::geometry_basis::GeometryBasis;
use crate::geometry::visual::geometry_visual::GeometryVisual;
use crate::geometry::visual::geometry_visual_vertex::GeometryVisualVertex;
use crate::light::point::light_color::LightColor;

impl GeometryVisual {
  /// `OGF::CalculateTB`, as a multi-use model's visuals take their basis: the mender run over the visual alone, each
  /// vertex it splits a copy of its source, the vertices ordered by first use before and after.
  ///
  /// # Errors
  ///
  /// Returns an error when the mended visual outgrows 16-bit indices.
  pub(crate) fn mend_basis(&mut self) -> XrfResult {
    // A mended vertex is a copy of a source with its place, normal and coordinate, whose light it keeps.
    let key = |position: &Vector3d, normal: &Vector3d, coordinate: &Vector2d| {
      [
        position.x,
        position.y,
        position.z,
        normal.x,
        normal.y,
        normal.z,
        coordinate.x,
        coordinate.y,
      ]
      .map(f32::to_bits)
    };
    let lights: HashMap<[u32; 8], LightColor> = self
      .vertices
      .iter()
      .map(|vertex| {
        (
          key(&vertex.position, &vertex.normal, &vertex.texture_coordinate),
          vertex.light,
        )
      })
      .collect();
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
        light: lights
          .get(&key(&vertex.position, &vertex.normal, &vertex.texture_coordinate))
          .copied()
          .unwrap_or_default(),
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
