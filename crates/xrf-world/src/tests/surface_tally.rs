use xrf_renderer::{RenderSurfaceGeometry, RenderSurfaceSpan};
use xrf_visual::{
  SectorDescription, SectorGeometry, SectorPackage, SectorSection, SectorSurface, VisualClusters, VisualDrawRange,
  VisualSection,
};

use crate::level::surface_tally::SurfaceTally;

fn section(byte_offset: u32, byte_length: u32) -> VisualSection {
  VisualSection {
    byte_offset,
    byte_length,
  }
}

fn drawn(shader_id: u16, drawables: Vec<u32>, start: u32) -> SectorSection {
  SectorSection {
    surface: SectorSurface {
      shader_id,
      ..Default::default()
    },
    drawables,
    draw: VisualDrawRange { start, count: 3 },
    bounds: None,
  }
}

/// Four vertices drawn as two triangles, each its own section of shader 7; vertex 1 carries a fraction in its tangent.
fn package() -> SectorPackage {
  let mut buffer: Vec<u8> = vec![0; 48];

  for (u, v) in [(0i16, 0i16), (1024, 0), (1024, 2048), (0, 512)] {
    buffer.extend_from_slice(&u.to_le_bytes());
    buffer.extend_from_slice(&v.to_le_bytes());
  }

  buffer.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 51, 0, 0, 0, 0, 0, 0, 0, 0]);
  buffer.extend_from_slice(&[0; 16]);

  for index in [0u32, 1, 2, 0, 2, 3] {
    buffer.extend_from_slice(&index.to_le_bytes());
  }

  SectorPackage {
    description: SectorDescription {
      sector: 0,
      geometry: SectorGeometry {
        vertex_count: 4,
        index_count: 6,
        positions: section(0, 48),
        normals: None,
        tangents: Some(section(64, 16)),
        binormals: Some(section(80, 16)),
        uvs: Some(section(48, 16)),
        uv_components: 2,
        lightmap_uvs: None,
        colors: None,
        indices: section(96, 24),
        clusters: VisualClusters {
          ranges: section(0, 0),
          spheres: section(0, 0),
        },
      },
      sections: vec![drawn(7, vec![1, 2], 0), drawn(7, vec![3], 3)],
      instances: Vec::new(),
      impostors: None,
      skipped: Vec::new(),
      bounds: None,
      buffer_length: buffer.len() as u32,
    },
    buffer,
  }
}

#[test]
fn surface_tally_counts_an_entry_across_its_sections() {
  let measured: Vec<RenderSurfaceGeometry> = SurfaceTally::measure(&package()).list();
  let fraction: f32 = (1024.0 + 51.0 / 255.0) / 1024.0;

  assert_eq!(measured.len(), 1);
  assert_eq!(measured[0].shader_id, 7);
  assert_eq!(measured[0].drawables, 3);
  assert_eq!(measured[0].triangles, 2);
  assert_eq!(
    measured[0].span,
    Some(RenderSurfaceSpan {
      u_min: 0.0,
      u_max: fraction,
      v_min: 0.0,
      v_max: 2.0,
    })
  );
  // The second triangle never reaches the vertex carrying the fraction, so it covers less.
  assert_eq!(
    measured[0].narrowest,
    Some(RenderSurfaceSpan {
      u_min: 0.0,
      u_max: 1.0,
      v_min: 0.0,
      v_max: 2.0,
    })
  );
}

#[test]
fn surface_tally_merges_sectors() {
  let mut tally: SurfaceTally = SurfaceTally::measure(&package());

  tally.merge(SurfaceTally::measure(&package()));

  let measured: Vec<RenderSurfaceGeometry> = tally.list();

  assert_eq!(measured[0].drawables, 6);
  assert_eq!(measured[0].triangles, 4);
}

#[test]
fn surface_tally_reads_no_coordinates_from_a_short_pack() {
  let mut short: SectorPackage = package();

  short.buffer.truncate(40);

  let measured: Vec<RenderSurfaceGeometry> = SurfaceTally::measure(&short).list();

  assert_eq!(measured[0].triangles, 2);
  assert_eq!(measured[0].span, None);
}
