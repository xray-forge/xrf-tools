use uuid::Uuid;
use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_level::{BuildAimapFile, BuildAimapNode, BuildAimapParams, LevelAiNode, LevelAiVersion};
use xrf_math::{Aabb, AiNodePosition, EPS_L, Vector3d};

use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_compilation::AiMapCompilation;
use crate::ai::ai_map_compiler::AiMapCompiler;
use crate::ai::ai_map_request::AiMapRequest;
use crate::ray::ray_tracing::RayTracing;
use crate::{AiMapCovers, AiMapGuid, RayBackend};

const NO_LINK: u32 = BuildAimapNode::NARROW_LINK_MASK;

// From xrAI's round trip of every plane value: one it keeps, one it moves a step down (`(0, 1, 110)` to
// `(0, 0, 109)`), and one on the fold's edge it moves to the other sign (`(1, 0, 127)` to `(1, 2, 127)`).
const PLANES: [(u16, u16); 3] = [
  ((4 << 13) | (1 << 7) | 79, (4 << 13) | (1 << 7) | 79),
  ((1 << 7) | 110, 109),
  ((1 << 13) | 127, (1 << 13) | (2 << 7) | 127),
];

fn node(x: i16, z: i16, y: u16, plane: u16, links: [u32; 4]) -> BuildAimapNode {
  BuildAimapNode {
    links,
    plane,
    position: AiNodePosition { x, y, z },
  }
}

// Out of the save's order, so the save sorts them.
pub(super) fn aimap() -> BuildAimapFile {
  BuildAimapFile {
    version: BuildAimapFile::NARROW_LINKS_VERSION,
    bounds: Aabb {
      min: Vector3d::new(1.4, -2.0, -0.7),
      max: Vector3d::new(2.1, 6.0, 0.7),
    },
    params: BuildAimapParams {
      patch_size: 0.7,
      test_height: 1.0,
      can_up: 1.5,
      can_down: 4.0,
    },
    nodes: vec![
      node(3, 1, 0, PLANES[0].0, [NO_LINK, 1, NO_LINK, NO_LINK]),
      node(3, 0, 65535, PLANES[1].0, [NO_LINK, 2, NO_LINK, 0]),
      node(3, -1, 32768, PLANES[2].0, [3, NO_LINK, NO_LINK, 1]),
      node(2, -1, 0, PLANES[0].0, [NO_LINK, NO_LINK, 2, NO_LINK]),
    ],
  }
}

fn on_cpu() -> RayTracing<'static> {
  RayTracing {
    backend: RayBackend::Cpu,
    dxc: None,
    gpu: None,
  }
}

fn compile(replaced: Option<&AiMap>) -> XrfResult<AiMapCompilation> {
  AiMapCompiler::compile(
    AiMapRequest {
      aimap: &aimap(),
      covers: AiMapCovers::None,
      scene: None,
      tracing: on_cpu(),
      replaced,
    },
    &JobHandle::inert(),
  )
}

fn nodes(map: &AiMap) -> Vec<LevelAiNode> {
  map.nodes.iter_nodes().collect()
}

#[test]
fn saves_the_nodes_sorted_by_cell_with_links_renumbered() -> XrfResult {
  let compiled: AiMapCompilation = compile(None)?;
  let header = &compiled.map.file.header;
  let saved: Vec<LevelAiNode> = nodes(&compiled.map);
  // Saved: the x 2 node, then the x 3 row by z: the input's 3, 2, 1, 0.
  let invalid: u32 = (1 << LevelAiVersion::CsCop.get_link_bits()) - 1;

  assert_eq!(header.version, LevelAiVersion::CsCop as u32);
  assert_eq!(header.count, 4);
  assert_eq!(header.size, 0.7);
  assert_eq!(header.size_y, 8.0 + EPS_L, "the box's height grown by EPS_L");
  assert_eq!(
    saved.iter().map(|node| node.links).collect::<Vec<[u32; 4]>>(),
    [
      [invalid, invalid, 1, invalid],
      [0, invalid, invalid, 2],
      [invalid, 1, invalid, 3],
      [invalid, 2, invalid, invalid],
    ]
  );
  assert_eq!(
    saved.iter().map(|node| node.packed_xz).collect::<Vec<u32>>(),
    [0, 3, 4, 5],
    "rows of three cells along z, from the nodes' own box"
  );
  assert_eq!(
    saved.iter().map(|node| node.packed_y).collect::<Vec<u16>>(),
    [0, 32763, 65526, 0],
    "heights over the grown extent"
  );
  assert!(
    saved
      .iter()
      .all(|node| node.high_cover == [0; 4] && node.low_cover == [0; 4] && node.light == 15)
  );

  Ok(())
}

#[test]
fn round_trips_each_plane_as_the_sdk_does() -> XrfResult {
  let saved: Vec<LevelAiNode> = nodes(&compile(None)?.map);

  assert_eq!(
    saved.iter().map(|node| node.plane).collect::<Vec<u16>>(),
    [PLANES[0].1, PLANES[2].1, PLANES[1].1, PLANES[0].1]
  );

  Ok(())
}

#[test]
fn derives_the_same_identity_from_the_same_nodes() -> XrfResult {
  let first: AiMapCompilation = compile(None)?;
  let second: AiMapCompilation = compile(None)?;

  assert_eq!(first.guid, AiMapGuid::Derived);
  assert_ne!(first.map.file.header.guid, Uuid::nil());
  assert_eq!(first.map.get_bytes()?, second.map.get_bytes()?);

  Ok(())
}

#[test]
fn keeps_the_replaced_identity_while_the_graph_stands() -> XrfResult {
  let mut replaced: AiMap = compile(None)?.map;
  let guid: Uuid = Uuid::from_u128(0x0123_4567_89AB_CDEF_0123_4567_89AB_CDEF);

  replaced.file.header.guid = guid;

  let kept: AiMapCompilation = compile(Some(&replaced))?;

  assert_eq!(kept.guid, AiMapGuid::Kept);
  assert_eq!(kept.map.file.header.guid, guid);

  // A link changed is another graph.
  let mut changed: Vec<LevelAiNode> = nodes(&replaced);

  changed[0].links[2] = (1 << LevelAiVersion::CsCop.get_link_bits()) - 1;
  replaced.nodes = xrf_level::LevelAiNodes::from_nodes(LevelAiVersion::CsCop, &changed)?;

  let derived: AiMapCompilation = compile(Some(&replaced))?;

  assert_eq!(derived.guid, AiMapGuid::Derived);
  assert_ne!(derived.map.file.header.guid, guid);

  Ok(())
}

#[test]
fn refuses_what_xrai_refuses_and_full_covers_with_no_ray_model() {
  let refuse = |aimap: BuildAimapFile, covers: AiMapCovers| {
    AiMapCompiler::compile(
      AiMapRequest {
        aimap: &aimap,
        covers,
        scene: None,
        tracing: on_cpu(),
        replaced: None,
      },
      &JobHandle::inert(),
    )
    .expect_err("the map is refused")
    .to_string()
  };

  assert!(refuse(aimap(), AiMapCovers::Full).contains("build.cform"));
  assert!(
    refuse(
      BuildAimapFile {
        version: BuildAimapFile::WIDE_LINKS_VERSION,
        ..aimap()
      },
      AiMapCovers::None
    )
    .contains("version 2")
  );
  assert!(
    refuse(
      BuildAimapFile {
        nodes: Vec::new(),
        ..aimap()
      },
      AiMapCovers::None
    )
    .contains("no node")
  );
}

#[test]
fn refuses_a_map_its_nodes_cannot_be_placed_or_walked_on() {
  let refuse = |aimap: BuildAimapFile| {
    AiMapCompiler::compile(
      AiMapRequest {
        aimap: &aimap,
        covers: AiMapCovers::None,
        scene: None,
        tracing: on_cpu(),
        replaced: None,
      },
      &JobHandle::inert(),
    )
    .expect_err("the map is refused")
    .to_string()
  };
  let sized = |patch_size: f32| BuildAimapFile {
    params: BuildAimapParams {
      patch_size,
      ..aimap().params
    },
    ..aimap()
  };
  let mut dangling: BuildAimapFile = aimap();
  let mut before_the_box: BuildAimapFile = aimap();

  dangling.nodes[1].links[0] = 4;
  before_the_box.nodes[3].position.x = -5;

  for size in [0.0, -0.7, f32::NAN, f32::INFINITY] {
    assert!(refuse(sized(size)).contains("cell size"), "a cell size of {size}");
  }

  assert!(refuse(dangling).contains("links to node 4"));
  assert!(refuse(before_the_box).contains("outside the map's cells"));
}
