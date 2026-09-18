//! What one sector becomes: one vertex array every drawable of it was moved onto, and one index array grouped by
//! the surface that draws it.

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelSectorComposition, LevelShadersChunk, LevelVisualsChunk};

use crate::data::sector_description::SectorDescription;
use crate::data::visual_submesh::VisualSkipCause;
use crate::pack::sector_package::SectorPackage;
use crate::pack::sector_packer::SectorPacker;
use crate::pack::tests::level_fixtures::{
  GeomBuffer, drawable, drawable_of_buffer, geometry, hierarchy, lightmapped_declaration, lightmapped_vertex,
  open_geometry, position_vertex, positions_declaration, shaders, tree, visuals,
};

/// Four lightmapped vertices in one buffer, and six indices that draw two triangles out of them.
fn new_geometry() -> Vec<u8> {
  geometry(
    &[GeomBuffer {
      declaration: lightmapped_declaration(),
      vertices: vec![
        lightmapped_vertex(0.0, 0.0, 1.0),
        lightmapped_vertex(1.0, 0.0, 2.0),
        lightmapped_vertex(0.0, 1.0, 3.0),
        lightmapped_vertex(1.0, 1.0, 4.0),
      ],
    }],
    &[0, 1, 0, 1, 1, 0],
  )
}

/// Everything one sector reaches, which is what the packer takes.
fn composition(run: &LevelVisualsChunk) -> LevelSectorComposition {
  LevelSectorComposition::of(run, 0)
}

/// The 32-bit indices a package wrote, read back out of its buffer.
fn read_indices(package: &SectorPackage) -> Vec<u32> {
  let section = package.description.indices;
  let start: usize = section.byte_offset as usize;

  package.buffer[start..start + section.byte_length as usize]
    .as_chunks::<4>()
    .0
    .iter()
    .map(|bytes| u32::from_le_bytes(*bytes))
    .collect()
}

/// The floats a section wrote, read back out of the buffer.
fn read_floats(package: &SectorPackage, section: crate::data::visual_section::VisualSection) -> Vec<f32> {
  let start: usize = section.byte_offset as usize;

  package.buffer[start..start + section.byte_length as usize]
    .as_chunks::<4>()
    .0
    .iter()
    .map(|bytes| f32::from_le_bytes(*bytes))
    .collect()
}

#[test]
fn test_packs_a_sector_into_one_buffer_of_parallel_arrays() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1, 2]), drawable(1, 0, 2, 0, 3), drawable(1, 2, 2, 3, 3)]);
  let table: LevelShadersChunk = shaders(&["", "default/stone,lmap"]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, Some(&table), &mut source).pack::<XRayByteOrder>(0, &composition(&run));
  let description: &SectorDescription = &package.description;

  assert_eq!(description.sector, 0);
  assert_eq!(
    description.vertex_count, 4,
    "each drawable brought its own two vertices"
  );
  assert_eq!(description.index_count, 6);
  assert_eq!(description.positions.byte_length, 4 * 3 * 4);
  assert_eq!(
    description
      .normals
      .expect("a lightmapped sector carries normals")
      .byte_length,
    4 * 3 * 4
  );
  assert_eq!(
    description
      .hemi
      .expect("the hemisphere term rides in the normal")
      .byte_length,
    4 * 4
  );
  assert!(description.skipped.is_empty());
  assert_eq!(description.buffer_length as usize, package.buffer.len());
}

#[test]
fn test_packs_a_range_two_drawables_share_only_once() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1, 2]), drawable(1, 0, 2, 0, 3), drawable(1, 0, 2, 0, 3)]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));

  assert_eq!(
    package.description.vertex_count, 2,
    "one range, packed once rather than once for each of the two drawables that name it"
  );
  assert_eq!(package.description.index_count, 6, "both drawables still draw");
}

// A stored index counts from its own visual's vertex base, because the renderer passes that base as the draw call's
// base vertex. Packing moves the range, so the base moves with it.
#[test]
fn test_moves_indices_onto_the_vertices_a_range_was_packed_at() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1, 2]), drawable(1, 0, 2, 0, 3), drawable(1, 2, 2, 3, 3)]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));

  // The first drawable stores 0, 1, 0 and the second 1, 1, 0; each is moved onto the vertices it was packed at,
  // and the winding of every triangle is reversed on the way in.
  assert_eq!(read_indices(&package), vec![0, 0, 1, 3, 2, 3]);
}

#[test]
fn test_groups_drawables_by_the_shader_entry_that_dresses_them() {
  let run: LevelVisualsChunk = visuals(&[
    hierarchy(&[1, 2, 3]),
    drawable(1, 0, 2, 0, 3),
    drawable(2, 0, 2, 0, 3),
    drawable(1, 2, 2, 3, 3),
  ]);
  let table: LevelShadersChunk = shaders(&[
    "",
    "def_shaders\\def_vertex/wall,wall_lm",
    "def_shaders\\def_aref/glass",
  ]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, Some(&table), &mut source).pack::<XRayByteOrder>(0, &composition(&run));
  let sections = &package.description.sections;

  assert_eq!(sections.len(), 2, "two surfaces, two draws");
  assert_eq!(sections[0].shader_id, 1);
  assert_eq!(
    sections[0].drawables,
    vec![1, 3],
    "both visuals of one surface draw together"
  );
  assert_eq!(sections[0].shader_name.as_deref(), Some("def_shaders\\def_vertex"));
  assert_eq!(sections[0].texture_name.as_deref(), Some("wall"));
  assert_eq!(sections[0].draw.start, 0);
  assert_eq!(sections[0].draw.count, 6);
  assert_eq!(sections[1].shader_id, 2);
  assert_eq!(
    sections[1].draw.start, 6,
    "one section begins where the one before it ended"
  );
  assert_eq!(sections[1].draw.count, 3);
}

#[test]
fn test_leaves_out_a_drawable_whose_range_it_cannot_read() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1, 2]), drawable(1, 0, 2, 0, 3), drawable(1, 3, 9, 0, 3)]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));

  assert_eq!(package.description.skipped.len(), 1);
  assert_eq!(package.description.skipped[0].drawable, 2);
  assert_eq!(package.description.skipped[0].cause, VisualSkipCause::Malformed);
  assert_eq!(
    package.description.vertex_count, 2,
    "the rest of the sector still packs"
  );
  assert_eq!(package.description.index_count, 3);
}

// One array holds an attribute for every vertex or for none, so a range that carries no lightmap coordinate still
// leaves room for one when another range of the same sector does.
#[test]
fn test_carries_an_attribute_any_range_of_the_sector_declares() {
  let bytes: Vec<u8> = geometry(
    &[
      GeomBuffer {
        declaration: lightmapped_declaration(),
        vertices: vec![lightmapped_vertex(0.0, 0.0, 0.0), lightmapped_vertex(1.0, 0.0, 0.0)],
      },
      GeomBuffer {
        declaration: positions_declaration(),
        vertices: vec![position_vertex(0.0, 1.0, 0.0), position_vertex(1.0, 1.0, 0.0)],
      },
    ],
    &[0, 1, 0],
  );
  let run: LevelVisualsChunk = visuals(&[
    hierarchy(&[1, 2]),
    drawable(1, 0, 2, 0, 3),
    drawable_of_buffer(1, 1, 0, 2, 0, 3),
  ]);
  let mut source = open_geometry(bytes);

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));
  let lightmap = package
    .description
    .lightmap_coordinates
    .expect("a sector one of whose ranges is lightmapped");

  assert_eq!(package.description.vertex_count, 4);
  assert_eq!(
    lightmap.byte_length,
    4 * 2 * 4,
    "every vertex of the sector, not only the lit ones"
  );

  let coordinates: Vec<f32> = read_floats(&package, lightmap);

  assert_eq!(
    &coordinates[4..],
    &[0.0, 0.0, 0.0, 0.0],
    "the range that carries none packs zeroes rather than a gap"
  );
}

#[test]
fn test_mirrors_the_level_into_renderer_space() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1]), drawable(1, 0, 2, 0, 3)]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));
  let positions: Vec<f32> = read_floats(&package, package.description.positions);

  assert_eq!(positions, vec![0.0, 0.0, -1.0, 1.0, 0.0, -2.0]);
}

#[test]
fn test_packs_a_sector_that_reaches_nothing_into_an_empty_package() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[])]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(7, &composition(&run));

  assert_eq!(package.description.sector, 7);
  assert_eq!(package.description.vertex_count, 0);
  assert_eq!(package.description.index_count, 0);
  assert!(package.description.sections.is_empty());
  assert!(package.description.bounds.is_none(), "nothing packed spans nothing");
}

// A tree keeps its mesh in its own space and carries the transform that stands it in the level. Packed as it is read,
// every tree of a level would be stacked at the origin.
#[test]
fn test_places_a_visual_stored_in_its_own_space() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1]), tree(1, 0, 2, 3, 100.0)]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));
  let positions: Vec<f32> = read_floats(&package, package.description.positions);

  // The fixture's first two vertices are at x 0 and 1, stood a hundred along x.
  assert_eq!(positions[0], 100.0);
  assert_eq!(positions[3], 101.0);
}

// Two trees naming one mesh are two trees. Sharing the range would pack it once and leave both of them standing in
// whichever place was read first.
#[test]
fn test_packs_an_instance_of_a_shared_range_for_each_place_it_stands() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1, 2]), tree(1, 0, 2, 3, 100.0), tree(1, 0, 2, 3, -100.0)]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));
  let positions: Vec<f32> = read_floats(&package, package.description.positions);

  assert_eq!(package.description.vertex_count, 4, "one mesh, two places, two copies");
  assert_eq!(positions[0], 100.0);
  assert_eq!(
    positions[6], -100.0,
    "the second instance stands where its own transform puts it"
  );
}

// A range already baked into the level is still shared, which is what the sharing was always for.
#[test]
fn test_still_shares_a_range_no_transform_places() {
  let run: LevelVisualsChunk = visuals(&[hierarchy(&[1, 2]), drawable(1, 0, 2, 0, 3), drawable(1, 0, 2, 0, 3)]);
  let mut source = open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, None, &mut source).pack::<XRayByteOrder>(0, &composition(&run));

  assert_eq!(package.description.vertex_count, 2);
}
