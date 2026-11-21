use std::collections::VecDeque;
use std::sync::Mutex;

use xrf_error::{XrfError, XrfResult};
use xrf_job::parallel;
use xrf_job::parallel::prelude::*;
use xrf_job::{JobHandle, JobScope};

use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_query::AiCoverQuery;
use crate::ai::cover::ai_node_covers::AiNodeCovers;
use crate::ai::cover::gpu::ai_cover_gpu_batch::AiCoverGpuBatch;
use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;
use crate::ray::ray_rules::RayRules;

/// The corner nodes' rays traced on a started GPU by xrAI's rules ([`RayRules::COVERS`]), answered as the CPU's pass
/// answers them. The CPU's threads flood and aim one chunk of nodes while the GPU traces the chunk before; covers are
/// summed on the CPU in the CPU pass's order.
pub(crate) struct AiCoverGpu<'a> {
  /// The GPU holding the cover pass's ray scene, started by `RayDevice`.
  pub(crate) tracer: &'a RayGpuTracer,
}

impl AiCoverGpu<'_> {
  /// Corner nodes flooded and aimed together.
  const NODES: usize = 256;

  /// Every corner node's covers, in the order `corners` lists them.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch, or the job is cancelled.
  pub(crate) fn trace_corners(
    &self,
    vertices: &[AiVertex],
    corners: &[u32],
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<Vec<AiNodeCovers>> {
    // One flood's marks a thread, kept across chunks: a node count's worth is too large to make again each time.
    let queries: Vec<Mutex<AiCoverQuery>> = (0..parallel::current_num_threads())
      .map(|_| Mutex::new(AiCoverQuery::new(vertices.len())))
      .collect();
    let aim = |nodes: &[u32]| Self::aim(vertices, nodes, &queries);
    let mut chunks = corners.chunks(Self::NODES);
    let mut waiting: VecDeque<AiCoverGpuBatch> = chunks.next().map(aim).transpose()?.into_iter().flatten().collect();
    let mut covers: Vec<AiNodeCovers> = Vec::with_capacity(corners.len());

    while let Some(batch) = waiting.pop_front() {
      job.check_cancelled()?;

      let next: Option<&[u32]> = if waiting.is_empty() { chunks.next() } else { None };
      let (traced, aimed) = parallel::join(
        || self.tracer.trace(&batch.rays, RayRules::COVERS),
        || next.map(aim).transpose(),
      );

      covers.extend(batch.sum(vertices, &traced?));
      scope.advance_by(batch.get_node_count() as u64);
      waiting.extend(aimed?.into_iter().flatten());
    }

    Ok(covers)
  }

  /// `nodes` flooded and their rays aimed, in batches one dispatch each takes.
  fn aim(vertices: &[AiVertex], nodes: &[u32], queries: &[Mutex<AiCoverQuery>]) -> XrfResult<Vec<AiCoverGpuBatch>> {
    let targets: Vec<Vec<u32>> = nodes
      .par_iter()
      .map(|node| {
        let mut query = queries[parallel::current_thread_index().unwrap_or(0) % queries.len()]
          .lock()
          .map_err(|_| XrfError::new_unexpected_error("An AI-map flood failed while holding its marks"))?;

        query.perform(vertices, *node);

        Ok(query.list.iter().copied().filter(|other| other != node).collect())
      })
      .collect::<XrfResult<_>>()?;

    AiCoverGpuBatch::pack(vertices, nodes, targets)
  }
}
