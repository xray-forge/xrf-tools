use std::path::Path;

use glam::Vec3;
use xrf_error::XrfResult;
use xrf_geometry::{BvhTracer, Tracer};
use xrf_job::JobHandle;
use xrf_math::{Aabb, Vector3d};

use crate::RayBackend;
use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_pass::AiCoverPass;
use crate::ai::cover::ai_cover_sector::AiCoverSector;
use crate::ai::cover::ai_cover_tracing::AiCoverTracing;
use crate::ai::cover::ai_cover_tree::AiCoverTree;
use crate::ai::cover::ai_cover_walk::AiCoverWalk;
use crate::ai::cover::ai_node_covers::AiNodeCovers;
use crate::ray::ray_scene::RayScene;
use crate::ray::ray_tracing::RayTracing;
use crate::shadow::shadow_materials::ShadowMaterials;

const SIDE: u32 = 5;

fn at(x: u32, z: u32) -> u32 {
  x * SIDE + z
}

fn grid(holes: &[u32]) -> Vec<AiVertex> {
  let link = |x: i64, z: i64| {
    let inside: bool = (0..i64::from(SIDE)).contains(&x) && (0..i64::from(SIDE)).contains(&z);

    match inside.then(|| at(x as u32, z as u32)) {
      Some(node) if !holes.contains(&node) => node,
      _ => AiVertex::NO_LINK,
    }
  };

  (0..SIDE * SIDE)
    .map(|node| {
      let (x, z) = (i64::from(node / SIDE), i64::from(node % SIDE));

      AiVertex {
        links: [link(x - 1, z), link(x, z + 1), link(x + 1, z), link(x, z - 1)],
        normal: Vector3d::new(0.0, 1.0, 0.0),
        position: Vector3d::new(x as f32, 0.0, z as f32),
        high_cover: [0.0; 4],
        low_cover: [0.0; 4],
      }
    })
    .collect()
}

// Its front faces -x.
pub(crate) fn wall() -> BvhTracer {
  BvhTracer::new(
    &[
      Vec3::new(2.5, 0.0, -1.0),
      Vec3::new(2.5, 0.0, 5.0),
      Vec3::new(2.5, 3.0, -1.0),
      Vec3::new(2.5, 3.0, 5.0),
    ],
    &[[0, 1, 2], [1, 3, 2]],
  )
}

pub(crate) fn covers(tracer: &BvhTracer, backend: RayBackend, dxc: Option<&Path>) -> XrfResult<Vec<AiNodeCovers>> {
  let shadows: ShadowMaterials = ShadowMaterials::new_opaque(tracer.get_face_count());
  let mut vertices: Vec<AiVertex> = grid(&[]);

  AiCoverPass::compute(
    &mut vertices,
    1.0,
    &AiCoverTracing {
      scene: RayScene {
        tracer,
        shadows: &shadows,
      },
      tracing: RayTracing {
        backend,
        dxc,
        gpu: None,
      },
    },
    &JobHandle::inert(),
  )?;

  Ok(
    vertices
      .iter()
      .map(|vertex| AiNodeCovers {
        high: vertex.high_cover,
        low: vertex.low_cover,
      })
      .collect(),
  )
}

#[test]
fn bins_directions_in_eight_sectors_a_vertical_one_in_the_sixth() {
  let sector = |x: f32, y: f32, z: f32| AiCoverSector::of(&Vector3d::new(x, y, z));

  assert_eq!(sector(1.0, 0.0, 0.5), 0);
  assert_eq!(sector(0.5, 0.0, 1.0), 1);
  assert_eq!(sector(-0.5, 0.0, 1.0), 2);
  assert_eq!(sector(-1.0, 0.0, 0.5), 3);
  assert_eq!(sector(-1.0, 0.0, -0.5), 4);
  assert_eq!(sector(-0.5, 0.0, -1.0), 5);
  assert_eq!(sector(0.5, 0.0, -1.0), 6);
  assert_eq!(sector(1.0, 0.0, -0.5), 7);
  // Straight along +x the ground's z is no more than zero.
  assert_eq!(sector(1.0, 0.0, 0.0), 7);
  assert_eq!(sector(0.0, 1.0, 0.0), 6);
  assert_eq!(
    AiCoverSector::get_covers(&[1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
    [0.0, 0.5, 0.75, 0.25]
  );
}

#[test]
fn takes_corners_not_straight_edges_for_cover_nodes() {
  let corners = |holes: &[u32]| {
    let vertices: Vec<AiVertex> = grid(holes);

    (0..SIDE * SIDE)
      .filter(|node| !holes.contains(node) && AiCoverPass::is_corner(&vertices, &vertices[*node as usize]))
      .collect::<Vec<u32>>()
  };

  assert_eq!(corners(&[]), [at(0, 0), at(0, 4), at(4, 0), at(4, 4)]);
  // A hole's four neighbours stand at inner corners: each misses a link a node beside it has.
  assert_eq!(
    corners(&[at(2, 2)]),
    [
      at(0, 0),
      at(0, 4),
      at(1, 2),
      at(2, 1),
      at(2, 3),
      at(3, 2),
      at(4, 0),
      at(4, 4)
    ]
  );
}

#[test]
fn walks_straight_toward_a_corner_until_a_hole_stops_it() {
  assert!(AiCoverWalk::is_reaching(&grid(&[]), at(2, 2), at(0, 0), 0.5));
  assert!(!AiCoverWalk::is_reaching(&grid(&[at(1, 1)]), at(2, 2), at(0, 0), 0.5));
  assert!(AiCoverWalk::is_reaching(&grid(&[]), at(4, 1), at(0, 4), 0.5));
}

#[test]
fn finds_points_within_reach_across_the_ground() {
  let positions: Vec<Vector3d> = (0..100)
    .map(|point| Vector3d::new((point / 10) as f32, (point % 3) as f32, (point % 10) as f32))
    .collect();
  let bounds: Aabb = Aabb::from_points(&positions).expect("points");
  let mut tree: AiCoverTree = AiCoverTree::new(&bounds, 0.5);

  for (point, position) in positions.iter().enumerate() {
    tree.insert(point as u32, position);
  }

  let center: Vector3d = Vector3d::new(4.2, 9.0, 5.1);
  let mut found: Vec<u32> = Vec::new();

  tree.find_nearest(&center, 1.5, &positions, &mut found);

  let mut sorted: Vec<u32> = found.clone();

  sorted.sort_unstable();
  sorted.dedup();

  assert_eq!(sorted.len(), found.len(), "each point listed once");
  assert!(found.contains(&45), "the nearest point");
  assert!(found.iter().all(|point| {
    let position: &Vector3d = &positions[*point as usize];

    (position.x - center.x).powi(2) + (position.z - center.z).powi(2) <= 1.5 * 1.5
  }));
}

#[test]
fn covers_corners_by_the_rays_a_wall_blocks_and_mixes_the_others() -> XrfResult {
  let open: Vec<AiNodeCovers> = covers(&BvhTracer::new(&[], &[]), RayBackend::Cpu, None)?;
  let walled: Vec<AiNodeCovers> = covers(&wall(), RayBackend::Cpu, None)?;
  const RIGHT: usize = 2;

  // From the open grid's corner, the nodes stand forward and right alone.
  assert_eq!(
    open[at(0, 0) as usize],
    AiNodeCovers {
      high: [0.0, 0.5, 0.75, 0.25],
      low: [0.0, 0.5, 0.75, 0.25],
    }
  );
  assert!(walled[at(0, 0) as usize].high[RIGHT] < open[at(0, 0) as usize].high[RIGHT]);
  assert!(walled[at(0, 0) as usize].low[RIGHT] < open[at(0, 0) as usize].low[RIGHT]);
  // The wall's back passes rays from beyond it.
  assert_eq!(walled[at(4, 0) as usize], open[at(4, 0) as usize]);
  // A node beside the wall mixes the corners' covers, the walled ones included.
  assert!(walled[at(1, 2) as usize].high[RIGHT] < open[at(1, 2) as usize].high[RIGHT]);
  assert!(walled.iter().chain(&open).all(|covers| {
    covers
      .high
      .iter()
      .chain(&covers.low)
      .all(|cover| (0.0..=1.0).contains(cover))
  }));

  Ok(())
}
