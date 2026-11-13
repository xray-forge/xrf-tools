use std::collections::VecDeque;
use std::sync::Mutex;

use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobScope};
use xrf_renderer_core::{RayQueryAsk, RayQueryDevice, RenderGpu};

use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_pass::AiNodeCovers;
use crate::ai::cover::ai_cover_query::AiCoverQuery;
use crate::ai::cover::ai_cover_tracing::AiCoverTracing;
use crate::ai::cover::gpu::ai_cover_gpu_batch::AiCoverGpuBatch;
use crate::detail::gpu::detail_gpu_tracer::DetailGpuTracer;

/// The corner nodes' rays traced on the GPU with the detail lighter's ray queries, which answer a ray as the CPU's
/// pass does: back faces passed, the first opaque face ending it, alpha faces dimming it. Nodes are flooded and their
/// rays aimed on the CPU's threads a chunk at a time, while the GPU traces the chunk before; the covers are summed on
/// the CPU in the CPU pass's order.
pub(crate) struct AiCoverGpu {
  tracer: DetailGpuTracer,
}

impl AiCoverGpu {
  /// Corner nodes flooded and aimed together.
  const NODES: usize = 256;

  /// The GPU `tracing` names started, its ray model on it.
  ///
  /// # Errors
  ///
  /// Returns an error when no GPU traces rays or the GPU refuses the ray model.
  pub(crate) fn start(tracing: &AiCoverTracing<'_>) -> XrfResult<Self> {
    let device: RayQueryDevice = RayQueryDevice::request(RayQueryAsk {
      dxc: tracing.dxc,
      gpu: tracing.gpu.map(RenderGpu::from),
    })?;

    Ok(Self {
      tracer: DetailGpuTracer::new(device, tracing.rays.tracer, tracing.rays.shadows)?,
    })
  }

  /// The name of the GPU tracing.
  pub(crate) fn get_name(&self) -> &str {
    self.tracer.get_name()
  }

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
    let queries: Vec<Mutex<AiCoverQuery>> = (0..rayon::current_num_threads())
      .map(|_| Mutex::new(AiCoverQuery::new(vertices.len())))
      .collect();
    let aim = |nodes: &[u32]| Self::aim(vertices, nodes, &queries);
    let mut chunks = corners.chunks(Self::NODES);
    let mut waiting: VecDeque<AiCoverGpuBatch> = chunks.next().map(aim).transpose()?.into_iter().flatten().collect();
    let mut covers: Vec<AiNodeCovers> = Vec::with_capacity(corners.len());

    while let Some(batch) = waiting.pop_front() {
      job.check_cancelled()?;

      let next: Option<&[u32]> = if waiting.is_empty() { chunks.next() } else { None };
      let (traced, aimed) = rayon::join(|| self.tracer.trace(&batch.rays), || next.map(aim).transpose());

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
        let mut query = queries[rayon::current_thread_index().unwrap_or(0) % queries.len()]
          .lock()
          .map_err(|_| XrfError::new_unexpected_error("An AI-map flood failed while holding its marks"))?;

        query.perform(vertices, *node);

        Ok(query.list.iter().copied().filter(|other| other != node).collect())
      })
      .collect::<XrfResult<_>>()?;

    AiCoverGpuBatch::pack(vertices, nodes, targets)
  }
}
