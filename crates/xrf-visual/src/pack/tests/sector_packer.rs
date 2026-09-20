//! What one sector becomes: one vertex array every drawable of it was moved onto, and one index array grouped by
//! the surface that draws it.

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelSectorComposition, LevelShadersChunk, LevelVisualsChunk};

use crate::data::sector_attributes::SectorAttributes;
use crate::data::sector_description::SectorDescription;
use crate::data::sector_instance_group::SectorInstanceGroup;
use crate::data::visual_submesh::VisualSkipCause;
use crate::pack::sector_package::SectorPackage;
use crate::pack::sector_packer::SectorPacker;
use crate::pack::tests::level_fixtures::{
  GeomBuffer, new_drawable, new_drawable_of_buffer, new_geometry_fixture, new_hierarchy, new_lightmapped_declaration,
  new_lightmapped_vertex, new_open_geometry, new_position_vertex, new_positions_declaration, new_shaders, new_tree,
  new_vertex_lit_declaration, new_vertex_lit_vertex, new_visuals,
};

/// Four lightmapped vertices in one buffer, and six indices that draw two triangles out of them.
fn new_geometry() -> Vec<u8> {
  new_geometry_fixture(
    &[GeomBuffer {
      declaration: new_lightmapped_declaration(),
      vertices: vec![
        new_lightmapped_vertex(0.0, 0.0, 1.0),
        new_lightmapped_vertex(1.0, 0.0, 2.0),
        new_lightmapped_vertex(0.0, 1.0, 3.0),
        new_lightmapped_vertex(1.0, 1.0, 4.0),
      ],
    }],
    &[0, 1, 0, 1, 1, 0],
  )
}

/// Everything one sector reaches, which is what the packer takes.
fn new_composition(run: &LevelVisualsChunk) -> LevelSectorComposition {
  LevelSectorComposition::of(run, 0)
}

/// The 32-bit indices a package wrote, read back out of its buffer.
fn new_read_indices(package: &SectorPackage) -> Vec<u32> {
  let section = package.description.geometry.indices;
  let start: usize = section.byte_offset as usize;

  package.buffer[start..start + section.byte_length as usize]
    .as_chunks::<4>()
    .0
    .iter()
    .map(|bytes| u32::from_le_bytes(*bytes))
    .collect()
}

/// The floats a section wrote, read back out of the buffer.
fn new_read_floats(package: &SectorPackage, section: crate::data::visual_section::VisualSection) -> Vec<f32> {
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
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable(1, 0, 2, 0, 3),
    new_drawable(1, 2, 2, 3, 3),
  ]);
  let table: LevelShadersChunk = new_shaders(&["", "default/stone,lmap"]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, Some(&table), &source).pack::<XRayByteOrder>(
    0,
    &new_composition(&run),
    SectorAttributes::all(),
  );
  let description: &SectorDescription = &package.description;

  assert_eq!(description.sector, 0);
  assert_eq!(
    description.geometry.vertex_count, 4,
    "each drawable brought its own two vertices"
  );
  assert_eq!(description.geometry.index_count, 6);
  assert_eq!(description.geometry.positions.byte_length, 4 * 3 * 4);
  assert_eq!(
    description
      .geometry
      .normals
      .expect("a lightmapped sector carries normals")
      .byte_length,
    4 * 3 * 4
  );
  assert_eq!(
    description
      .geometry
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
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable(1, 0, 2, 0, 3),
    new_drawable(1, 0, 2, 0, 3),
  ]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());

  assert_eq!(
    package.description.geometry.vertex_count, 2,
    "one range, packed once rather than once for each of the two drawables that name it"
  );
  assert_eq!(package.description.geometry.index_count, 6, "both drawables still draw");
}

// A stored index counts from its own visual's vertex base, because the renderer passes that base as the draw call's
// base vertex. Packing moves the range, so the base moves with it.
#[test]
fn test_moves_indices_onto_the_vertices_a_range_was_packed_at() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable(1, 0, 2, 0, 3),
    new_drawable(1, 2, 2, 3, 3),
  ]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());

  // The first drawable stores 0, 1, 0 and the second 1, 1, 0; each is moved onto the vertices it was packed at,
  // and the winding of every triangle is reversed on the way in.
  assert_eq!(new_read_indices(&package), vec![0, 0, 1, 3, 2, 3]);
}

#[test]
fn test_groups_drawables_by_the_shader_entry_that_dresses_them() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2, 3]),
    new_drawable(1, 0, 2, 0, 3),
    new_drawable(2, 0, 2, 0, 3),
    new_drawable(1, 2, 2, 3, 3),
  ]);
  let table: LevelShadersChunk = new_shaders(&[
    "",
    "def_shaders\\def_vertex/wall,wall_lm",
    "def_shaders\\def_aref/glass",
  ]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage = SectorPacker::new(&run, Some(&table), &source).pack::<XRayByteOrder>(
    0,
    &new_composition(&run),
    SectorAttributes::all(),
  );
  let sections = &package.description.sections;

  assert_eq!(sections.len(), 2, "two surfaces, two draws");
  assert_eq!(sections[0].surface.shader_id, 1);
  assert_eq!(
    sections[0].drawables,
    vec![1, 3],
    "both visuals of one surface draw together"
  );
  assert_eq!(
    sections[0].surface.shader_name.as_deref(),
    Some("def_shaders\\def_vertex")
  );
  assert_eq!(sections[0].surface.texture_name.as_deref(), Some("wall"));
  assert_eq!(sections[0].draw.start, 0);
  assert_eq!(sections[0].draw.count, 6);
  assert_eq!(sections[1].surface.shader_id, 2);
  assert_eq!(
    sections[1].draw.start, 6,
    "one section begins where the one before it ended"
  );
  assert_eq!(sections[1].draw.count, 3);
}

#[test]
fn test_leaves_out_a_drawable_whose_range_it_cannot_read() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable(1, 0, 2, 0, 3),
    new_drawable(1, 3, 9, 0, 3),
  ]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());

  assert_eq!(package.description.skipped.len(), 1);
  assert_eq!(package.description.skipped[0].drawable, 2);
  assert_eq!(package.description.skipped[0].cause, VisualSkipCause::Malformed);
  assert_eq!(
    package.description.geometry.vertex_count, 2,
    "the rest of the sector still packs"
  );
  assert_eq!(package.description.geometry.index_count, 3);
}

// One array holds an attribute for every vertex or for none, so a range that carries no lightmap coordinate still
// leaves room for one when another range of the same sector does.
#[test]
fn test_carries_an_attribute_any_range_of_the_sector_declares() {
  let bytes: Vec<u8> = new_geometry_fixture(
    &[
      GeomBuffer {
        declaration: new_lightmapped_declaration(),
        vertices: vec![
          new_lightmapped_vertex(0.0, 0.0, 0.0),
          new_lightmapped_vertex(1.0, 0.0, 0.0),
        ],
      },
      GeomBuffer {
        declaration: new_positions_declaration(),
        vertices: vec![new_position_vertex(0.0, 1.0, 0.0), new_position_vertex(1.0, 1.0, 0.0)],
      },
    ],
    &[0, 1, 0],
  );
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable(1, 0, 2, 0, 3),
    new_drawable_of_buffer(1, 1, 0, 2, 0, 3),
  ]);
  let source = new_open_geometry(bytes);

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());
  let lightmap = package
    .description
    .geometry
    .lightmap_uvs
    .expect("a sector one of whose ranges is lightmapped");

  assert_eq!(package.description.geometry.vertex_count, 4);
  assert_eq!(
    lightmap.byte_length,
    4 * 2 * 4,
    "every vertex of the sector, not only the lit ones"
  );

  let coordinates: Vec<f32> = new_read_floats(&package, lightmap);

  assert_eq!(
    &coordinates[4..],
    &[0.0, 0.0, 0.0, 0.0],
    "the range that carries none packs zeroes rather than a gap"
  );
}

#[test]
fn test_mirrors_the_level_into_renderer_space() {
  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[1]), new_drawable(1, 0, 2, 0, 3)]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());
  let positions: Vec<f32> = new_read_floats(&package, package.description.geometry.positions);

  assert_eq!(positions, vec![0.0, 0.0, -1.0, 1.0, 0.0, -2.0]);
}

#[test]
fn test_packs_a_sector_that_reaches_nothing_into_an_empty_package() {
  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[])]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(7, &new_composition(&run), SectorAttributes::all());

  assert_eq!(package.description.sector, 7);
  assert_eq!(package.description.geometry.vertex_count, 0);
  assert_eq!(package.description.geometry.index_count, 0);
  assert!(package.description.sections.is_empty());
  assert!(package.description.bounds.is_none(), "nothing packed spans nothing");
}

// A tree keeps its mesh in its own space and carries the transform that stands it in the level, so it is packed as a
// mesh plus the places it stands rather than as geometry of its own.
#[test]
fn test_packs_a_visual_stored_in_its_own_space_as_an_instance() {
  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[1]), new_tree(1, 0, 2, 3, 100.0)]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());

  assert_eq!(
    package.description.geometry.vertex_count, 0,
    "a placed visual is not baked into the sector"
  );
  assert_eq!(package.description.instances.len(), 1);

  let group: &SectorInstanceGroup = &package.description.instances[0];

  assert_eq!(group.instance_count, 1);
  assert_eq!(group.geometry.vertex_count, 2, "the mesh itself, in its own space");
  assert_eq!(group.drawables, vec![1]);

  // Row major with the translation in the fourth row, mirrored into renderer space along the way.
  let transforms: Vec<f32> = new_read_floats(&package, group.transforms);

  assert_eq!(transforms.len(), SectorInstanceGroup::FLOATS_PER_INSTANCE);
  assert_eq!(transforms[12], 100.0);
}

// The whole point: one mesh in many places is one mesh and many places. Packing a copy each is what made a sector of
// a swamp cost a gigabyte.
#[test]
fn test_packs_one_mesh_for_every_place_it_stands() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_tree(1, 0, 2, 3, 100.0),
    new_tree(1, 0, 2, 3, -100.0),
  ]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());

  assert_eq!(package.description.instances.len(), 1, "one mesh");

  let group: &SectorInstanceGroup = &package.description.instances[0];

  assert_eq!(group.instance_count, 2, "two places");
  assert_eq!(group.geometry.vertex_count, 2, "packed once, not once for each place");
  assert_eq!(group.drawables, vec![1, 2]);

  let transforms: Vec<f32> = new_read_floats(&package, group.transforms);

  assert_eq!(transforms.len(), 2 * SectorInstanceGroup::FLOATS_PER_INSTANCE);
  assert_eq!(transforms[12], 100.0);
  assert_eq!(transforms[12 + SectorInstanceGroup::FLOATS_PER_INSTANCE], -100.0);
}

// Two meshes dressed by different surfaces cannot share one instanced draw, whatever else they have in common.
#[test]
fn test_keeps_instances_of_different_surfaces_apart() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_tree(1, 0, 2, 3, 100.0),
    new_tree(2, 0, 2, 3, -100.0),
  ]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());

  assert_eq!(package.description.instances.len(), 2);
  assert_eq!(package.description.instances[0].surface.shader_id, 1);
  assert_eq!(package.description.instances[1].surface.shader_id, 2);
}

// A range already baked into the level is still shared, which is what the sharing was always for.
#[test]
fn test_still_shares_a_range_no_transform_places() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable(1, 0, 2, 0, 3),
    new_drawable(1, 0, 2, 0, 3),
  ]);
  let source = new_open_geometry(new_geometry());

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());

  assert_eq!(package.description.geometry.vertex_count, 2);
  assert!(package.description.instances.is_empty());
}

// The colour channel multiplies, so the neutral value for a vertex that carries none is white. Filling those with
// black instead told a renderer every lightmapped and every tree surface of the sector was unlit, and a level
// drawing its baked colour came out a silhouette.
#[test]
fn test_widens_a_missing_vertex_colour_to_white_rather_than_black() {
  let bytes: Vec<u8> = new_geometry_fixture(
    &[
      GeomBuffer {
        declaration: new_vertex_lit_declaration(),
        vertices: vec![new_vertex_lit_vertex(0.0, 0.0, 0.0, [16, 32, 64, 255])],
      },
      GeomBuffer {
        declaration: new_lightmapped_declaration(),
        vertices: vec![new_lightmapped_vertex(1.0, 0.0, 0.0)],
      },
    ],
    &[0, 0, 0],
  );
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable(1, 0, 1, 0, 3),
    new_drawable_of_buffer(1, 1, 0, 1, 0, 3),
  ]);
  let source = new_open_geometry(bytes);

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());
  let colors = package
    .description
    .geometry
    .colors
    .expect("a sector one of whose ranges is vertex lit");
  let values: Vec<f32> = new_read_floats(&package, colors);

  assert_eq!(
    values.len(),
    6,
    "three components for each of the sector's two vertices"
  );
  // Written blue, green, red: the vertex lit one keeps what the compiler baked.
  assert_eq!(&values[0..3], &[64.0 / 255.0, 32.0 / 255.0, 16.0 / 255.0]);
  // The lightmapped one carries no colour and must contribute nothing to the multiply.
  assert_eq!(&values[3..6], &[1.0, 1.0, 1.0]);
}

// A sector carries whatever its declarations together carry; a caller that samples none of it should not be sent it.
// On marsh the tangent basis and the hemisphere term are 69 of the sector's 254 megabytes, paid on the wire and held
// at both ends.
#[test]
fn test_packs_only_the_attributes_the_caller_draws_with() {
  let bytes: Vec<u8> = new_geometry();
  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[1]), new_drawable(0, 0, 4, 0, 6)]);
  let source = new_open_geometry(bytes);

  let wanted: SectorAttributes = SectorAttributes {
    normals: true,
    uvs: true,
    ..SectorAttributes::default()
  };

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), wanted);
  let geometry = &package.description.geometry;

  assert!(geometry.normals.is_some(), "what the caller asked for is packed");
  assert!(geometry.uvs.is_some());
  assert!(
    geometry.tangents.is_none() && geometry.binormals.is_none(),
    "a tangent basis the caller never samples is not written"
  );
  assert!(
    geometry.hemi.is_none(),
    "nor a hemisphere term, which rides in the normal"
  );
  assert!(
    geometry.lightmap_uvs.is_none(),
    "the declaration carries a lightmap coordinate and the caller did not ask for it"
  );
}

// The counterpart: asking for everything leaves what the sector declares untouched, so no caller loses an attribute
// by not knowing to name it.
#[test]
fn test_packs_everything_a_sector_declares_for_a_caller_that_wants_it_all() {
  let bytes: Vec<u8> = new_geometry();
  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[1]), new_drawable(0, 0, 4, 0, 6)]);
  let source = new_open_geometry(bytes);

  let package: SectorPackage =
    SectorPacker::new(&run, None, &source).pack::<XRayByteOrder>(0, &new_composition(&run), SectorAttributes::all());
  let geometry = &package.description.geometry;

  assert!(geometry.normals.is_some() && geometry.tangents.is_some() && geometry.binormals.is_some());
  assert!(geometry.uvs.is_some() && geometry.lightmap_uvs.is_some() && geometry.hemi.is_some());
}
