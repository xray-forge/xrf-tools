use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{Ray, Tracer};
use xrf_job::parallel::prelude::*;
use xrf_job::{JobHandle, JobScope};
use xrf_math::glam::Vec3;
use xrf_math::{Aabb, Vector3d};

use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_query::AiCoverQuery;
use crate::ai::cover::ai_cover_sector::AiCoverSector;
use crate::ai::cover::ai_cover_tracing::AiCoverTracing;
use crate::ai::cover::ai_cover_tree::AiCoverTree;
use crate::ai::cover::ai_cover_walk::AiCoverWalk;
use crate::ai::cover::ai_node_covers::AiNodeCovers;
use crate::ray::ray_device::RayDevice;
use crate::ray::ray_rules::RayRules;
use crate::ray::ray_scene::RayScene;

/// xrAI's covers (`xrCover`, `xrAI/compiler_cover.cpp`), in single precision and the same order: each corner node
/// traces rays toward every node it reaches within 30 m, from a standing and a crouching height; every other node mixes
/// the covers of the corner nodes it walks straight to within 30 m, weighted by nearness.
///
/// Unlike xrAI, every ray is a full query, never first tested against the face that last blocked one, so the covers do
/// not depend on order or threads.
pub(crate) struct AiCoverPass;

impl AiCoverPass {
  /// What a job reports tracing under: the nodes at the map's corners.
  pub(crate) const PROGRESS_CORNERS: &'static str = "corner nodes";

  /// What a job reports mixing under: the other nodes.
  pub(crate) const PROGRESS_OTHERS: &'static str = "other nodes";

  /// The heights a corner node's rays leave from for its high and low covers, `high_cover_height` and
  /// `low_cover_height`.
  pub(crate) const HEIGHTS: [f32; 2] = [1.5, 0.6];

  /// `cover_distance`, how far a node's covers look across the ground.
  const REACH: f32 = 30.0;

  /// A cover summed over its four directions from which a node counts as open, `4 * .999f`.
  const OPEN: f32 = 4.0 * 0.999;

  /// Computes every node's covers, the map's cells `cell_size` across, tracing as `rays` says; returns the name of the
  /// GPU that traced, if one did.
  ///
  /// # Errors
  ///
  /// Returns [`XrfError::Cancelled`] when the job is cancelled, leaving the covers unfinished, and an error when the
  /// GPU backend cannot trace or fails. `Auto` falls back to the CPU only where the GPU does not start.
  pub(crate) fn compute(
    vertices: &mut [AiVertex],
    cell_size: f32,
    rays: &AiCoverTracing<'_>,
    job: &JobHandle,
  ) -> XrfResult<Option<String>> {
    let is_corner: Vec<bool> = vertices
      .iter()
      .map(|vertex| Self::is_corner(vertices, vertex))
      .collect();
    let corners: Vec<u32> = (0..vertices.len() as u32)
      .filter(|node| is_corner[*node as usize])
      .collect();
    let scope: JobScope = job.enter(Self::PROGRESS_CORNERS, Some(corners.len() as u64));
    let device: RayDevice = RayDevice::start(&rays.tracing, rays.scene, "the AI map's cover rays")?;
    let traced: Vec<AiNodeCovers> = Self::trace_on(&device, vertices, &corners, rays.scene, job, &scope)?;

    for (node, AiNodeCovers { high, low }) in corners.iter().zip(traced) {
      vertices[*node as usize].high_cover = high;
      vertices[*node as usize].low_cover = low;
    }

    let mixed: Vec<Option<AiNodeCovers>> = Self::mix_others(vertices, &is_corner, &corners, cell_size, job)?;

    for (vertex, covers) in vertices.iter_mut().zip(mixed) {
      if let Some(AiNodeCovers { high, low }) = covers {
        vertex.high_cover = high;
        vertex.low_cover = low;
      }
    }

    Ok(device.get_name())
  }

  /// `is_cover`: whether a node stands at a corner of the map, with a link missing on a side where a link
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

  /// The ray from `height` over `base` toward the point `height` over `target`, its direction and range as
  /// `compute_cover_value` divides them, and the sector it falls in.
  pub(crate) fn aim(base: &Vector3d, target: &Vector3d, height: f32) -> (Ray, usize) {
    let offset: Vector3d = target - base;
    let range: f32 = offset.length();
    let direction: Vector3d = &offset / range;

    (
      Ray::new(Vec3::new(base.x, base.y + height, base.z), Vec3::from(direction), range),
      AiCoverSector::of(&direction),
    )
  }

  /// One height's four covers from its rays' sectors and transmittances, in the order the rays were aimed: each
  /// sector's share passed, an empty sector 0.
  pub(crate) fn sum_covers(rays: impl Iterator<Item = (usize, f32)>) -> [f32; 4] {
    let mut total: [f32; 8] = [0.0; 8];
    let mut passed: [f32; 8] = [0.0; 8];

    for (sector, transmittance) in rays {
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

  /// Every corner node's covers, their rays traced on `device`, which holds `scene` where it is a GPU.
  fn trace_on(
    device: &RayDevice,
    vertices: &[AiVertex],
    corners: &[u32],
    scene: RayScene<'_>,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<Vec<AiNodeCovers>> {
    #[cfg(feature = "gpu")]
    if let Some(tracer) = device.get_gpu() {
      return crate::ai::cover::gpu::ai_cover_gpu::AiCoverGpu { tracer }.trace_corners(vertices, corners, job, scope);
    }

    #[cfg(not(feature = "gpu"))]
    let _ = device;

    Self::trace_on_cpu(vertices, corners, scene, job, scope)
  }

  /// Every corner node's covers, each whole on a thread of the installed pool, every ray a query of the CPU's tracer.
  fn trace_on_cpu(
    vertices: &[AiVertex],
    corners: &[u32],
    scene: RayScene<'_>,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<Vec<AiNodeCovers>> {
    let RayScene { tracer, shadows } = scene;

    corners
      .par_iter()
      .map_init(
        || AiCoverQuery::new(vertices.len()),
        |query, node| {
          job.check_cancelled()?;
          query.perform(vertices, *node);

          let base: &Vector3d = &vertices[*node as usize].position;
          let trace = |height: f32| {
            Self::sum_covers(query.list.iter().filter(|other| **other != *node).map(|other| {
              let (ray, sector) = Self::aim(base, &vertices[*other as usize].position, height);
              let mut transmittance: f32 = 1.0;

              tracer.for_each_hit(&ray, |hit| {
                transmittance *= RayRules::COVERS.get_transmittance(shadows.get_opacity(&hit));
              });

              (sector, transmittance)
            }))
          };
          let covers: AiNodeCovers = AiNodeCovers {
            high: trace(Self::HEIGHTS[0]),
            low: trace(Self::HEIGHTS[1]),
          };

          scope.advance();

          Ok(covers)
        },
      )
      .collect()
  }

  /// `compute_non_covers`: each other node's covers, `None` for a corner node, mixed from the corner nodes not
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
      return AiNodeCovers {
        high: [1.0; 4],
        low: [1.0; 4],
      };
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

    AiNodeCovers {
      high: high.map(|cover| cover.clamp(0.0, 1.0)),
      low: low.map(|cover| cover.clamp(0.0, 1.0)),
    }
  }

  fn sum(covers: &[f32; 4]) -> f32 {
    covers[0] + covers[1] + covers[2] + covers[3]
  }
}
