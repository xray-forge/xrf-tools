use xrf_visual::{SectorGeometry, VisualClusters, VisualSection};

use crate::scene::static_scene::geometry::geometry_clusters::GeometryClusters;

// A range a damaged mesh ends in one or two indices past its last whole triangle is cut into clusters of its whole
// triangles alone: no cluster of none, which the rows' cull and draw would count back from nought.
#[test]
fn cuts_a_range_into_its_whole_triangles_alone() {
  let positions: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
  let indices: [u32; 5] = [0, 1, 2, 0, 1];
  let mut buffer: Vec<u8> = Vec::new();
  let mut section = |bytes: &[u8]| -> VisualSection {
    let byte_offset: u32 = buffer.len() as u32;

    buffer.extend_from_slice(bytes);

    VisualSection {
      byte_offset,
      byte_length: bytes.len() as u32,
    }
  };
  let positions: VisualSection = section(bytemuck::cast_slice(&positions));
  let indices: VisualSection = section(bytemuck::cast_slice(&indices));
  let empty: VisualSection = VisualSection {
    byte_offset: buffer.len() as u32,
    byte_length: 0,
  };
  let geometry: SectorGeometry = SectorGeometry {
    vertex_count: 3,
    index_count: 5,
    positions,
    normals: None,
    tangents: None,
    binormals: None,
    uvs: None,
    uv_components: 0,
    lightmap_uvs: None,
    colors: None,
    indices,
    clusters: VisualClusters {
      ranges: empty,
      spheres: empty,
    },
  };
  let clusters: GeometryClusters<'_> = GeometryClusters::new(&geometry, &buffer);

  let cut: Vec<(u32, u32, glam::Vec4)> = clusters.cut(0, 5);

  assert_eq!(cut.len(), 1);
  assert_eq!((cut[0].0, cut[0].1), (0, 1));
  assert!(clusters.cut(0, 2).is_empty(), "no whole triangle, no cluster");
}
