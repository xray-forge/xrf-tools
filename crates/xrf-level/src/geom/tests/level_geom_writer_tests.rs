use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::level_geom_buffers::LevelGeomBuffers;
use crate::geom::level_geom_packer::LevelGeomPacker;
use crate::geom::level_geom_range::LevelGeomRange;
use crate::geom::vertex::level_vertex::LevelVertex;
use crate::geom::vertex::level_vertex_base_coordinate::LevelVertexBaseCoordinate;
use crate::geom::vertex::level_vertex_declaration::LevelVertexDeclaration;
use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;
use crate::geom::vertex::level_vertex_payload::LevelVertexPayload;
use crate::geom::vertex::level_vertex_quantizer::LevelVertexQuantizer;
use crate::geom::vertex::level_vertex_stored::LevelVertexStored;
use crate::geom::window::level_geom_slide_window::LevelGeomSlideWindow;
use crate::geom::window::level_geom_slide_window_item::LevelGeomSlideWindowItem;
use crate::level::level_header_chunk::LevelHeaderChunk;

fn layout_of(declaration: Vec<LevelGeomVertexElement>) -> XrfResult<LevelVertexLayout> {
  LevelVertexLayout::of_declaration(&declaration)
}

fn vertex_lit(position: f32) -> LevelVertexStored {
  LevelVertexStored {
    position: Vector3d::new(position, 2.0, 3.0),
    normal: Some([1, 2, 3, 4]),
    tangent: Some([5, 6, 7, 8]),
    binormal: Some([9, 10, 11, 12]),
    color: Some([13, 14, 15, 16]),
    texture_coordinate: Some([-17, 18, 0, 0]),
    lightmap_coordinate: None,
  }
}

#[test]
fn writes_a_vertex_where_each_layout_places_its_elements_and_reads_it_back() -> XrfResult {
  for (declaration, stored) in [
    (LevelVertexDeclaration::get_vertex_lit(), vertex_lit(1.0)),
    (
      LevelVertexDeclaration::get_lightmapped(),
      LevelVertexStored {
        color: None,
        lightmap_coordinate: Some([-19, 20]),
        ..vertex_lit(1.0)
      },
    ),
    (
      LevelVertexDeclaration::get_fast_path(),
      LevelVertexStored {
        position: Vector3d::new(1.0, 2.0, 3.0),
        normal: None,
        tangent: None,
        binormal: None,
        color: None,
        texture_coordinate: None,
        lightmap_coordinate: None,
      },
    ),
  ] {
    let layout: LevelVertexLayout = layout_of(declaration)?;
    let mut bytes: Vec<u8> = vec![0; layout.get_stride() as usize];

    stored.write_into::<XRayByteOrder>(&layout, &mut bytes)?;

    let payload: LevelVertexPayload = LevelVertexPayload::of(layout, bytes)?;

    assert_eq!(
      payload
        .vertices()
        .next()
        .map(|vertex| vertex.get_stored::<XRayByteOrder>()),
      Some(stored)
    );
  }

  assert_eq!(layout_of(LevelVertexDeclaration::get_vertex_lit())?.get_stride(), 32);
  assert_eq!(layout_of(LevelVertexDeclaration::get_fast_path())?.get_stride(), 12);

  Ok(())
}

#[test]
fn refuses_a_vertex_whose_elements_its_layout_does_not_hold() -> XrfResult {
  let layout: LevelVertexLayout = layout_of(LevelVertexDeclaration::get_lightmapped())?;
  let mut bytes: Vec<u8> = vec![0; layout.get_stride() as usize];

  assert!(
    vertex_lit(1.0)
      .write_into::<XRayByteOrder>(&layout, &mut bytes)
      .is_err()
  );
  assert!(
    vertex_lit(1.0)
      .write_into::<XRayByteOrder>(&layout, &mut bytes[1..])
      .is_err()
  );

  Ok(())
}

#[test]
fn quantizes_as_xrlc_rounds_down_and_clamps() {
  assert_eq!(
    LevelVertexQuantizer::get_direction(&Vector3d::new(1.0, 0.0, -1.0), 7),
    // Blue, green, red, extra: z = -1 to 0, y = 0 to 127, x = 1 to 255.
    [0, 127, 255, 7]
  );
  assert_eq!(
    LevelVertexQuantizer::get_direction(&Vector3d::new(2.0, -2.0, 0.0), 0)[LevelVertex::DIRECTION_X],
    255
  );
  assert_eq!(
    LevelVertexQuantizer::get_base_coordinate(1.5),
    LevelVertexBaseCoordinate {
      primary: 1536,
      fraction: 0
    }
  );
  // A quarter of a step past 1/1024: the byte below takes 255.5 of a step.
  assert_eq!(
    LevelVertexQuantizer::get_base_coordinate(1.25 / 1024.0),
    LevelVertexBaseCoordinate {
      primary: 1,
      fraction: 63
    }
  );
  assert_eq!(
    LevelVertexQuantizer::get_base_coordinate(-40.0).primary,
    i16::MIN,
    "past 32 tiles clamps"
  );
  assert_eq!(LevelVertexQuantizer::get_lightmap_coordinate(0.5), 16384);
  assert_eq!(LevelVertexQuantizer::get_lightmap_coordinate(1.0), i16::MAX);
  assert_eq!(LevelVertexQuantizer::get_light_term(0.5), 127);
  assert_eq!(LevelVertexQuantizer::get_light_term(1.0), 255);
  assert_eq!(LevelVertexQuantizer::get_light_term(-0.5), 0);
}

#[test]
fn packs_visuals_into_the_first_buffer_of_their_declaration_they_fit() -> XrfResult {
  let lit = |count: usize| vec![0u8; count * 32];
  let mut packer: LevelGeomPacker = LevelGeomPacker::default();

  assert_eq!(
    packer.add_vertices(&LevelVertexDeclaration::get_vertex_lit(), &lit(10))?,
    LevelGeomRange { buffer: 0, start: 0 }
  );
  // Another declaration starts its own buffer; the first takes more of its own until the vertex limit.
  assert_eq!(
    packer.add_vertices(&LevelVertexDeclaration::get_fast_path(), &[0; 12])?,
    LevelGeomRange { buffer: 1, start: 0 }
  );
  assert_eq!(
    packer.add_vertices(&LevelVertexDeclaration::get_vertex_lit(), &lit(65525))?,
    LevelGeomRange { buffer: 0, start: 10 }
  );
  assert_eq!(
    packer.add_vertices(&LevelVertexDeclaration::get_vertex_lit(), &lit(1))?,
    LevelGeomRange { buffer: 2, start: 0 }
  );
  assert!(
    packer
      .add_vertices(&LevelVertexDeclaration::get_vertex_lit(), &[0; 31])
      .is_err()
  );
  assert!(
    packer
      .add_vertices(&LevelVertexDeclaration::get_vertex_lit(), &[])
      .is_err()
  );

  // An index buffer holds fewer than its limit.
  assert_eq!(packer.add_indices(&[0, 1, 2])?, LevelGeomRange { buffer: 0, start: 0 });
  assert_eq!(
    packer.add_indices(&vec![0; LevelGeomPacker::INDEX_LIMIT - 3])?,
    LevelGeomRange { buffer: 1, start: 0 }
  );
  assert_eq!(packer.add_indices(&[3, 4, 5])?, LevelGeomRange { buffer: 0, start: 3 });

  Ok(())
}

#[test]
fn writes_a_geometry_file_that_reads_back_whole() -> XrfResult {
  let layout: LevelVertexLayout = layout_of(LevelVertexDeclaration::get_vertex_lit())?;
  let mut packer: LevelGeomPacker = LevelGeomPacker::default();
  let mut bytes: Vec<u8> = vec![0; 64];

  vertex_lit(1.0).write_into::<XRayByteOrder>(&layout, &mut bytes[..32])?;
  vertex_lit(2.0).write_into::<XRayByteOrder>(&layout, &mut bytes[32..])?;
  packer.add_vertices(&LevelVertexDeclaration::get_vertex_lit(), &bytes)?;
  packer.add_indices(&[0, 1, 1])?;

  let buffers: LevelGeomBuffers = packer.into_buffers(
    LevelHeaderChunk {
      xrlc_version: 14,
      xrlc_quality: 0,
    },
    vec![LevelGeomSlideWindowItem {
      reserved: [1, 2, 3, 4],
      windows: vec![LevelGeomSlideWindow {
        offset: 0,
        triangles: 1,
        vertices: 2,
      }],
    }],
  );
  let mut writer: ChunkWriter = ChunkWriter::new();

  buffers.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(
    LevelGeomBuffers::read_from_bytes::<XRayByteOrder>(writer.flush_raw_into_buffer()?)?,
    buffers
  );

  Ok(())
}
