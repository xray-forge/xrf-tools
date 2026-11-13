use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_pass::{AiCoverPass, AiNodeCovers};
use crate::detail::gpu::detail_gpu_batch::DetailGpuBatch;
use crate::detail::gpu::detail_gpu_records::GpuSingle;
use crate::detail::gpu::detail_gpu_tracer::DetailGpuTracer;

/// Some corner nodes' rays as the GPU traces them: each node's targets, and its rays from each height in turn, one
/// numbered run, a node's after the node before.
pub(crate) struct AiCoverGpuBatch {
  nodes: Vec<u32>,
  /// Each node's targets, itself left out, in the flood's order.
  targets: Vec<Vec<u32>>,
  /// Each node's first ray's number.
  firsts: Vec<usize>,
  pub(crate) rays: DetailGpuBatch,
}

impl AiCoverGpuBatch {
  /// Batches of `nodes`, each `targets` theirs, as few as one dispatch each takes.
  ///
  /// # Errors
  ///
  /// Returns an error for a node with more rays than one dispatch takes.
  pub(crate) fn pack(vertices: &[AiVertex], nodes: &[u32], targets: Vec<Vec<u32>>) -> XrfResult<Vec<Self>> {
    let heights: usize = AiCoverPass::HEIGHTS.len();
    let mut batches: Vec<Self> = Vec::new();
    let mut taken: Vec<(u32, Vec<u32>)> = Vec::new();
    let mut count: usize = 0;

    for (node, list) in nodes.iter().zip(targets) {
      let rays: usize = list.len() * heights;

      if rays > DetailGpuTracer::MOST_SINGLES {
        return Err(XrfError::new_invalid_error(format!(
          "AI-map node {node} traces {rays} rays, more than one GPU dispatch takes"
        )));
      }

      if count + rays > DetailGpuTracer::MOST_SINGLES {
        batches.push(Self::new(vertices, std::mem::take(&mut taken)));
        count = 0;
      }

      count += rays;
      taken.push((*node, list));
    }

    if !taken.is_empty() {
      batches.push(Self::new(vertices, taken));
    }

    Ok(batches)
  }

  /// The batch of `taken`'s nodes and targets, its rays aimed on the installed pool.
  fn new(vertices: &[AiVertex], taken: Vec<(u32, Vec<u32>)>) -> Self {
    let (nodes, targets): (Vec<u32>, Vec<Vec<u32>>) = taken.into_iter().unzip();
    let mut firsts: Vec<usize> = Vec::with_capacity(nodes.len());
    let mut count: usize = 0;

    for list in &targets {
      firsts.push(count);
      count += list.len() * AiCoverPass::HEIGHTS.len();
    }

    let singles: Vec<GpuSingle> = nodes
      .par_iter()
      .zip(&targets)
      .zip(&firsts)
      .flat_map_iter(|((node, list), first)| {
        let base: Vector3d = vertices[*node as usize].position;

        AiCoverPass::HEIGHTS
          .iter()
          .enumerate()
          .flat_map(move |(height_index, height)| {
            list.iter().enumerate().map(move |(target, other)| {
              let (ray, _) = AiCoverPass::aim(&base, &vertices[*other as usize].position, *height);

              GpuSingle::new(&ray, (first + height_index * list.len() + target) as u32)
            })
          })
      })
      .collect();

    Self {
      nodes,
      targets,
      firsts,
      rays: DetailGpuBatch {
        singles,
        ray_count: count,
        ..DetailGpuBatch::default()
      },
    }
  }

  /// How many corner nodes the batch holds.
  pub(crate) fn get_node_count(&self) -> usize {
    self.nodes.len()
  }

  /// Each node's covers from its rays' `transmittances`, by their numbers, summed on the installed pool.
  pub(crate) fn sum(&self, vertices: &[AiVertex], transmittances: &[f32]) -> Vec<AiNodeCovers> {
    self
      .nodes
      .par_iter()
      .zip(&self.targets)
      .zip(&self.firsts)
      .map(|((node, list), first)| {
        let base: &Vector3d = &vertices[*node as usize].position;
        let covers = |height_index: usize| {
          let height: f32 = AiCoverPass::HEIGHTS[height_index];

          AiCoverPass::sum_covers(list.iter().enumerate().map(|(target, other)| {
            let (_, sector) = AiCoverPass::aim(base, &vertices[*other as usize].position, height);

            (sector, transmittances[first + height_index * list.len() + target])
          }))
        };

        (covers(0), covers(1))
      })
      .collect()
  }
}
