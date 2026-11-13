use glam::Vec3;
use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{Ray, Tracer};
use xrf_job::{JobHandle, JobScope};
use xrf_math::{Aabb, Vector3d};

use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_query::AiCoverQuery;
use crate::ai::cover::ai_cover_sector::AiCoverSector;
use crate::ai::cover::ai_cover_tree::AiCoverTree;
use crate::ai::cover::ai_cover_walk::AiCoverWalk;
use crate::ai::cover::ai_ray_model::AiRayModel;

/// xrAI's covers (`xrCover`, `xrAI/compiler_cover.cpp:567`), its arithmetic in single precision in the same order:
/// each node at a corner of the map traced against the ray model toward every node it reaches within 30 m, from a
/// standing and a crouching height; every other node then given the covers of the corner nodes it walks straight to
/// within 30 m, weighted by nearness. Unlike xrAI, every ray is a full query of the ray model, never a test of the face
/// that last blocked a ray toward the same node first, so the covers are the same whatever the order and threads.
pub(crate) struct AiCoverPass;

/// One node's high and low covers.
pub(crate) type AiNodeCovers = ([f32; 4], [f32; 4]);

impl AiCoverPass {
  /// What a job reports tracing under: the nodes at the map's corners.
  pub(crate) const PROGRESS_CORNERS: &'static str = "corner nodes";

  /// What a job reports mixing under: the other nodes.
  pub(crate) const PROGRESS_OTHERS: &'static str = "other nodes";

  /// `high_cover_height`, a standing eye.
  const HIGH: f32 = 1.5;

  /// `low_cover_height`, a crouching eye.
  const LOW: f32 = 0.6;

  /// `cover_distance`, how far a node's covers look across the ground.
  const REACH: f32 = 30.0;

  /// A cover summed over its four directions from which a node counts as open, `4 * .999f`.
  const OPEN: f32 = 4.0 * 0.999;

  /// Computes every node's covers, the map's cells `cell_size` across.
  ///
  /// # Errors
  ///
  /// Returns [`XrfError::Cancelled`] when the job is cancelled, the covers then left unfinished.
  pub(crate) fn compute(
    vertices: &mut [AiVertex],
    cell_size: f32,
    rays: &AiRayModel<'_>,
    job: &JobHandle,
  ) -> XrfResult<()> {
    let is_corner: Vec<bool> = vertices
      .iter()
      .map(|vertex| Self::is_corner(vertices, vertex))
      .collect();
    let corners: Vec<u32> = (0..vertices.len() as u32)
      .filter(|node| is_corner[*node as usize])
      .collect();
    let traced: Vec<AiNodeCovers> = Self::trace_corners(vertices, &corners, rays, job)?;

    for (node, (high, low)) in corners.iter().zip(traced) {
      vertices[*node as usize].high_cover = high;
      vertices[*node as usize].low_cover = low;
    }

    let mixed: Vec<Option<AiNodeCovers>> = Self::mix_others(vertices, &is_corner, &corners, cell_size, job)?;

    for (vertex, covers) in vertices.iter_mut().zip(mixed) {
      if let Some((high, low)) = covers {
        vertex.high_cover = high;
        vertex.low_cover = low;
      }
    }

    Ok(())
  }

  /// `is_cover` (`:364`): whether a node stands at a corner of the map, with a link missing on a side where a link
  /// across it is missing too, or where a neighbour across it reaches round to that side.
  pub(crate) fn is_corner(vertices: &[AiVertex], vertex: &AiVertex) -> bool {
    let is_linked = |link: u32| link != AiVertex::NO_LINK;
    // Whether the node's link `first` has a link `second` itself.
    let is_round = |first: usize, second: usize| {
      is_linked(vertex.links[first]) && is_linked(vertices[vertex.links[first] as usize].links[second])
    };
    let is_critical = |side: usize, one: usize, other: usize| {
      !is_linked(vertex.links[side])
        && (!is_linked(vertex.links[one])
          || !is_linked(vertex.links[other])
          || is_round(one, side)
          || is_round(other, side))
    };

    is_critical(0, 1, 3) || is_critical(2, 1, 3) || is_critical(1, 0, 2) || is_critical(3, 0, 2)
  }

  /// Every corner node's covers, the nodes shared over the installed pool.
  fn trace_corners(
    vertices: &[AiVertex],
    corners: &[u32],
    rays: &AiRayModel<'_>,
    job: &JobHandle,
  ) -> XrfResult<Vec<AiNodeCovers>> {
    let scope: JobScope = job.enter(Self::PROGRESS_CORNERS, Some(corners.len() as u64));

    corners
      .par_iter()
      .map_init(
        || AiCoverQuery::new(vertices.len()),
        |query, node| {
          job.check_cancelled()?;
          query.perform(vertices, *node);

          let covers: AiNodeCovers = (
            Self::trace(vertices, *node, &query.list, Self::HIGH, rays),
            Self::trace(vertices, *node, &query.list, Self::LOW, rays),
          );

          scope.advance();

          Ok(covers)
        },
      )
      .collect()
  }

  /// `compute_cover_value` (`:248`): one corner node's covers from `height` over it, by the share of each sector's rays
  /// toward the nodes `reached` lists that the ray model lets through.
  fn trace(vertices: &[AiVertex], node: u32, reached: &[u32], height: f32, rays: &AiRayModel<'_>) -> [f32; 4] {
    let base: &Vector3d = &vertices[node as usize].position;
    let eye: Vec3 = Vec3::new(base.x, base.y + height, base.z);
    let mut total: [f32; 8] = [0.0; 8];
    let mut passed: [f32; 8] = [0.0; 8];

    for other in reached.iter().filter(|other| **other != node) {
      let offset: Vector3d = &vertices[*other as usize].position - base;
      let range: f32 = offset.length();
      let direction: Vector3d = &offset / range;
      let sector: usize = AiCoverSector::of(&direction);
      let mut transmittance: f32 = 1.0;

      rays.tracer.for_each_hit(
        &Ray::new(eye, Vec3::new(direction.x, direction.y, direction.z), range),
        |hit| transmittance *= rays.shadows.get_transmittance(&hit),
      );

      total[sector] += 1.0;
      passed[sector] += transmittance;
    }

    let values: [f32; 8] = std::array::from_fn(|sector| {
      if total[sector] == 0.0 {
        0.0
      } else {
        (passed[sector] / total[sector]).clamp(0.0, 1.0)
      }
    });

    AiCoverSector::get_covers(&values)
  }

  /// `compute_non_covers` (`:448`): each other node's covers, `None` for a corner node, mixed from the corner nodes not
  /// open both ways that it walks straight to within reach, each weighted by one over its distance; all open where it
  /// reaches none.
  fn mix_others(
    vertices: &[AiVertex],
    is_corner: &[bool],
    corners: &[u32],
    cell_size: f32,
    job: &JobHandle,
  ) -> XrfResult<Vec<Option<AiNodeCovers>>> {
    let bounds: Aabb = Aabb::from_points(vertices.iter().map(|vertex| vertex.position))
      .ok_or_else(|| XrfError::new_invalid_error("Level AI-map holds no node to cover"))?;
    let mut tree: AiCoverTree = AiCoverTree::new(&bounds, cell_size * 0.5);
    let positions: Vec<Vector3d> = vertices.iter().map(|vertex| vertex.position).collect();

    for node in corners {
      let vertex: &AiVertex = &vertices[*node as usize];

      if Self::sum(&vertex.high_cover) < Self::OPEN || Self::sum(&vertex.low_cover) < Self::OPEN {
        tree.insert(*node, &vertex.position);
      }
    }

    let scope: JobScope = job.enter(Self::PROGRESS_OTHERS, Some((vertices.len() - corners.len()) as u64));

    (0..vertices.len())
      .into_par_iter()
      .map_init(Vec::new, |nearest, node| {
        if is_corner[node] {
          return Ok(None);
        }

        job.check_cancelled()?;
        tree.find_nearest(&positions[node], Self::REACH, &positions, nearest);

        let covers: AiNodeCovers = Self::mix(vertices, node as u32, nearest, cell_size);

        scope.advance();

        Ok(Some(covers))
      })
      .collect()
  }

  /// One other node's covers from the corner nodes `nearest` lists, in that order.
  fn mix(vertices: &[AiVertex], node: u32, nearest: &[u32], cell_size: f32) -> AiNodeCovers {
    let position: &Vector3d = &vertices[node as usize].position;
    let mut total: f32 = 0.0;
    let weights: Vec<(f32, u32)> = nearest
      .iter()
      .filter(|corner| AiCoverWalk::is_reaching(vertices, node, **corner, cell_size * 0.5))
      .map(|corner| {
        let weight: f32 = 1.0 / vertices[*corner as usize].position.distance(position);

        total += weight;

        (weight, *corner)
      })
      .collect();

    if weights.is_empty() {
      return ([1.0; 4], [1.0; 4]);
    }

    let mut high: [f32; 4] = [0.0; 4];
    let mut low: [f32; 4] = [0.0; 4];

    for (weight, corner) in weights {
      let factor: f32 = weight / total;
      let vertex: &AiVertex = &vertices[corner as usize];

      for direction in 0..4 {
        high[direction] += factor * vertex.high_cover[direction];
        low[direction] += factor * vertex.low_cover[direction];
      }
    }

    (
      high.map(|cover| cover.clamp(0.0, 1.0)),
      low.map(|cover| cover.clamp(0.0, 1.0)),
    )
  }

  fn sum(covers: &[f32; 4]) -> f32 {
    covers[0] + covers[1] + covers[2] + covers[3]
  }
}
