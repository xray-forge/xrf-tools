//! A sector of trees for the tests of tree types and their baked impostors: two one-triangle trees of one type, the
//! first composed by the sector's one impostor.

use glam::{Mat4, Vec3, Vec4};
use xrf_visual::{
  SectorDescription, SectorGeometry, SectorImpostorGroup, SectorImpostors, SectorInstanceGroup, SectorPackage,
  SectorSurface, VisualClusters, VisualSection,
};

/// The type the sector's one impostor names.
pub(crate) const TREE_KIND: u32 = 5;

/// Where its two trees stand, both of that type, the first in the impostor's clump.
pub(crate) const TREES: [Vec3; 2] = [Vec3::new(10.0, 0.0, 0.0), Vec3::new(-10.0, 0.0, 5.0)];

fn surface(shader: &str) -> SectorSurface {
  SectorSurface {
    shader_id: 0,
    shader_name: Some(String::from(shader)),
    texture_name: Some(String::from("test")),
    hemi: None,
  }
}

/// Appends `values` to the package's buffer, answering where they stand in it.
fn append<T: bytemuck::Pod>(buffer: &mut Vec<u8>, values: &[T]) -> VisualSection {
  let byte_offset: u32 = buffer.len() as u32;

  buffer.extend_from_slice(bytemuck::cast_slice(values));

  VisualSection {
    byte_offset,
    byte_length: buffer.len() as u32 - byte_offset,
  }
}

/// The sector: its trees a triangle two metres high and two across, standing upright in its own `z = 0` plane.
pub(crate) fn create_tree_sector() -> SectorPackage {
  let mut buffer: Vec<u8> = Vec::new();
  let positions: VisualSection = append(&mut buffer, &[[-1.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 2.0, 0.0]]);
  let indices: VisualSection = append(&mut buffer, &[0u32, 1, 2]);
  let transforms: VisualSection = append(&mut buffer, &TREES.map(|at| Mat4::from_translation(at).to_cols_array()));
  let hemi: VisualSection = append(&mut buffer, &[1.0f32, 0.0, 1.0, 0.0]);
  let lods: VisualSection = append(&mut buffer, &[0i32, SectorInstanceGroup::NO_IMPOSTOR]);
  let spheres: VisualSection = append(&mut buffer, &[TREES[0].extend(3.0)]);
  let factors: VisualSection = append(&mut buffer, &[1.0f32]);
  let corners: VisualSection = append(
    &mut buffer,
    &[TREES[0].extend(0.0).to_array(); SectorImpostors::CORNERS * 2],
  );
  let normals: VisualSection = append(&mut buffer, &[Vec4::X; 8]);
  let types: VisualSection = append(&mut buffer, &[TREE_KIND]);
  let empty: VisualSection = VisualSection {
    byte_offset: buffer.len() as u32,
    byte_length: 0,
  };
  let geometry =
    |vertex_count: u32, index_count: u32, positions: VisualSection, indices: VisualSection| SectorGeometry {
      vertex_count,
      index_count,
      positions,
      normals: None,
      tangents: None,
      binormals: None,
      uvs: None,
      // A tree's four: its coordinate, then its rigidity.
      uv_components: if index_count == 0 { 0 } else { 4 },
      lightmap_uvs: None,
      colors: None,
      indices,
      clusters: VisualClusters {
        ranges: empty,
        spheres: empty,
      },
    };

  SectorPackage {
    description: SectorDescription {
      sector: 0,
      geometry: geometry(0, 0, empty, empty),
      sections: Vec::new(),
      instances: vec![SectorInstanceGroup {
        surface: surface("default"),
        drawables: Vec::new(),
        geometry: geometry(3, 3, positions, indices),
        instance_count: 2,
        transforms,
        hemi,
        impostors: Some(lods),
        progressive: None,
      }],
      impostors: Some(SectorImpostors {
        groups: vec![SectorImpostorGroup {
          surface: surface("details\\lod"),
          start: 0,
          count: 1,
        }],
        spheres,
        factors,
        corners,
        normals,
        types,
      }),
      skipped: Vec::new(),
      bounds: None,
      buffer_length: buffer.len() as u32,
    },
    buffer,
  }
}
