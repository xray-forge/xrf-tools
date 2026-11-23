use xrf_visual::{
  SectorDescription, SectorGeometry, SectorPackage, SectorSection, SectorSurface, VisualClusters, VisualDrawRange,
  VisualSection,
};

/// Sector `sector` of two sections: a ground quad two metres across at the origin, and a triangle five metres up ten
/// along.
pub(super) fn create_sector(sector: u32) -> SectorPackage {
  let positions: [[f32; 3]; 7] = [
    [-1.0, 0.0, -1.0],
    [1.0, 0.0, -1.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [9.0, 5.0, -1.0],
    [11.0, 5.0, -1.0],
    [10.0, 5.0, 1.0],
  ];
  let indices: [u32; 9] = [0, 1, 2, 0, 2, 3, 4, 5, 6];
  let mut buffer: Vec<u8> = bytemuck::cast_slice(&positions).to_vec();
  let index_offset: u32 = buffer.len() as u32;

  buffer.extend_from_slice(bytemuck::cast_slice(&indices));

  let end: u32 = buffer.len() as u32;
  let section = |start: u32, count: u32| SectorSection {
    surface: SectorSurface {
      shader_id: 0,
      shader_name: Some(String::from("default")),
      texture_name: Some(String::from("test")),
      hemi: None,
    },
    drawables: Vec::new(),
    draw: VisualDrawRange { start, count },
    bounds: None,
  };

  SectorPackage {
    description: SectorDescription {
      sector,
      geometry: SectorGeometry {
        vertex_count: positions.len() as u32,
        index_count: indices.len() as u32,
        positions: VisualSection {
          byte_offset: 0,
          byte_length: index_offset,
        },
        normals: None,
        tangents: None,
        binormals: None,
        uvs: None,
        uv_components: 0,
        lightmap_uvs: None,
        colors: None,
        indices: VisualSection {
          byte_offset: index_offset,
          byte_length: end - index_offset,
        },
        clusters: VisualClusters {
          ranges: VisualSection {
            byte_offset: end,
            byte_length: 0,
          },
          spheres: VisualSection {
            byte_offset: end,
            byte_length: 0,
          },
        },
      },
      sections: vec![section(0, 6), section(6, 3)],
      instances: Vec::new(),
      impostors: None,
      skipped: Vec::new(),
      bounds: None,
      buffer_length: end,
    },
    buffer,
  }
}
