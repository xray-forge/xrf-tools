use xrf_chunk::XRayByteOrder;
use xrf_level::LevelVisualsChunk;

use crate::pack::tests::sector::level_fixtures::{
  GeomBuffer, new_geometry_fixture, new_hierarchy, new_open_geometry, new_shaders, new_tree, new_tree_declaration,
  new_tree_vertex, new_visuals,
};
use crate::pack::tree::tree_part::TreePart;
use crate::pack::tree::tree_type::TreeType;
use crate::pack::tree::tree_types::TreeTypes;

// A trunk (shader 0, vertices from 0) and a crown (shader 1, vertices from 4) of four triangles and two; a bush (shader
// 2, vertices from 8) of one.
fn trunk(at: f32) -> Vec<u8> {
  new_tree(0, 0, 4, 12, at)
}

fn crown(at: f32) -> Vec<u8> {
  new_tree(1, 4, 4, 6, at)
}

fn bush(at: f32) -> Vec<u8> {
  new_tree(2, 8, 3, 3, at)
}

// The parts standing at one transform are one tree, and every tree of the same parts one type, however many there are
// and wherever they stand: two whole trees, a bush, and a crown standing alone make three types.
#[test]
fn test_groups_a_levels_trees_by_the_parts_standing_together() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2, 3, 4, 5, 6]),
    trunk(0.0),
    crown(0.0),
    crown(10.0),
    trunk(10.0),
    bush(20.0),
    crown(30.0),
  ]);
  let trees: TreeTypes = TreeTypes::of(&run);
  let trunk_part: TreePart = TreePart {
    visual: 1,
    shader_id: 0,
    triangles: 4,
  };
  let crown_part: TreePart = TreePart {
    visual: 2,
    shader_id: 1,
    triangles: 2,
  };

  assert_eq!((trees.parts, trees.placements), (6, 4));
  assert_eq!(trees.types.len(), 3);
  assert_eq!(
    trees.types[0],
    TreeType {
      parts: vec![trunk_part, crown_part],
      instances: 2,
    },
    "the whole tree, stood twice, first"
  );
  assert_eq!(trees.types[0].get_triangles(), 6);
  assert!(trees.types[1..].iter().all(|kind| kind.instances == 1));
  assert!(
    trees.types[1..].iter().any(|kind| kind.parts == [crown_part]),
    "a crown alone is a type of its own"
  );
  assert!(
    trees.types[1..]
      .iter()
      .any(|kind| kind.parts.len() == 1 && kind.parts[0].shader_id == 2)
  );
}

// One mesh dressed by two shaders draws two parts, so trees of each are two types.
#[test]
fn test_keeps_a_mesh_dressed_differently_a_type_of_its_own() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_tree(0, 0, 4, 12, 0.0),
    new_tree(3, 0, 4, 12, 10.0),
  ]);
  let trees: TreeTypes = TreeTypes::of(&run);

  assert_eq!(trees.types.len(), 2);
  assert_eq!((trees.parts, trees.placements), (2, 2));
}

/// The key of a level's one tree type: a tree of two vertices stored from `base` of its buffer, dressed by `shader`, the
/// entry `slot` of its shader table.
fn read_one_key(base: u32, (slot, shader): (u16, &str)) -> u64 {
  let mut vertices: Vec<Vec<u8>> = (0..base).map(|at| new_tree_vertex(at as f32, 5.0, 5.0)).collect();
  let mut entries: Vec<&str> = vec![""; usize::from(slot)];

  vertices.extend([new_tree_vertex(0.0, 0.0, 0.0), new_tree_vertex(1.0, 0.0, 0.0)]);
  entries.push(shader);

  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[1]), new_tree(slot, base, 2, 3, 0.0)]);
  let source = new_open_geometry(new_geometry_fixture(
    &[GeomBuffer {
      declaration: new_tree_declaration(),
      vertices,
    }],
    &[0, 1, 0],
  ));
  let keys: Vec<u64> = TreeTypes::of(&run)
    .read_keys::<XRayByteOrder, _>(&run, Some(&new_shaders(&entries)), &source)
    .expect("the tree's keys");

  assert_eq!(keys.len(), 1);

  keys[0]
}

// A type keys by what draws it, not where the level stores it: the same tree stored elsewhere and named by another slot
// of the shader table keys the same, and the same mesh dressed by another shader keys apart.
#[test]
fn test_keys_a_type_by_its_content_across_levels() {
  let pine: u64 = read_one_key(0, (1, "trees\\trees/pine"));

  assert_eq!(read_one_key(3, (2, "trees\\trees/pine")), pine);
  assert_ne!(read_one_key(0, (1, "trees\\trees/birch")), pine);
}

#[test]
fn test_finds_no_type_in_a_level_without_trees() {
  assert_eq!(TreeTypes::of(&new_visuals(&[new_hierarchy(&[])])), TreeTypes::default());
}
