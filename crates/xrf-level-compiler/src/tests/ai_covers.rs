//! The AI map's covers, as xrAI's `xrCover` computes them.

use std::path::Path;

use glam::Vec3;
use xrf_error::XrfResult;
use xrf_geometry::{BvhTracer, Tracer};
use xrf_job::JobHandle;
use xrf_math::{Aabb, Vector3d};

use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_pass::{AiCoverPass, AiNodeCovers};
use crate::ai::cover::ai_cover_sector::AiCoverSector;
use crate::ai::cover::ai_cover_tracing::AiCoverTracing;
use crate::ai::cover::ai_cover_tree::AiCoverTree;
use crate::ai::cover::ai_cover_walk::AiCoverWalk;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::{AiRayModel, RayBackend};

const SIDE: u32 = 5;

/// A node's index in a [`SIDE`] by [`SIDE`] grid, by column along x and row along z.
fn at(x: u32, z: u32) -> u32 {
  x * SIDE + z
}

/// A flat grid of 1 m cells, every node linked to its neighbours, less the nodes `holes` names.
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

/// A wall 3 m high across the grid between x 2 and x 3, its front toward -x.
pub(super) fn wall() -> BvhTracer {
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

/// Each node of the grid's covers once computed against `tracer`, every face blocking, its corner rays traced on
/// `backend`, with `dxc` for D3D12.
pub(super) fn covers(tracer: &BvhTracer, backend: RayBackend, dxc: Option<&Path>) -> XrfResult<Vec<AiNodeCovers>> {
  let shadows: ShadowMaterials = ShadowMaterials::new_opaque(tracer.get_face_count());
  let mut vertices: Vec<AiVertex> = grid(&[]);

  AiCoverPass::compute(
    &mut vertices,
    1.0,
    &AiCoverTracing {
      rays: AiRayModel {
        tracer,
        shadows: &shadows,
      },
      backend,
      dxc,
      gpu: None,
    },
    &JobHandle::inert(),
  )?;

  Ok(
    vertices
      .iter()
      .map(|vertex| (vertex.high_cover, vertex.low_cover))
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
    ([0.0, 0.5, 0.75, 0.25], [0.0, 0.5, 0.75, 0.25])
  );
  assert!(walled[at(0, 0) as usize].0[RIGHT] < open[at(0, 0) as usize].0[RIGHT]);
  assert!(walled[at(0, 0) as usize].1[RIGHT] < open[at(0, 0) as usize].1[RIGHT]);
  // The wall's back passes rays from beyond it.
  assert_eq!(walled[at(4, 0) as usize], open[at(4, 0) as usize]);
  // A node beside the wall mixes the corners' covers, the walled ones included.
  assert!(walled[at(1, 2) as usize].0[RIGHT] < open[at(1, 2) as usize].0[RIGHT]);
  assert!(
    walled
      .iter()
      .chain(&open)
      .all(|(high, low)| { high.iter().chain(low).all(|cover| (0.0..=1.0).contains(cover)) })
  );

  Ok(())
}
