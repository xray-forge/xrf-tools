use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;

use crate::ai::ai_map_compilation::AiMapCompilation;
use crate::ai::ai_map_covers::AiMapCovers;
use crate::ai::ai_map_request::AiMapRequest;
use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_pass::AiCoverPass;
use crate::ai::cover::ai_cover_tracing::AiCoverTracing;

/// Compiles a level's AI map, `build.aimap` into `level.ai`, as SDK 0.7's `xrAI -f` does (`xrCompiler`,
/// `xrAI/compiler.cpp:63`): the nodes loaded, their covers computed, and the map saved sorted by cell. Node lighting is
/// not computed: xrAI's is dead code, and no loader reads the value.
pub struct AiMapCompiler;

impl AiMapCompiler {
  /// What a job reports tracing the covers under: the nodes at the map's corners.
  pub const PROGRESS_CORNER_NODES: &'static str = AiCoverPass::PROGRESS_CORNERS;

  /// What a job reports mixing the covers under: the other nodes.
  pub const PROGRESS_OTHER_NODES: &'static str = AiCoverPass::PROGRESS_OTHERS;

  /// Compiles the map `request` names, its covers shared over the installed pool and the corner nodes' rays traced by
  /// the request's backend.
  ///
  /// # Errors
  ///
  /// Returns an error when `build.aimap` holds no node or is not the version xrAI loads, when a node does not fit the
  /// file's layout, or for full covers with no ray model; [`XrfError::Cancelled`] when the job is cancelled; and, for
  /// the GPU backend, an error when this build has none, no GPU traces rays, or the GPU fails. `Auto` traces on the CPU
  /// instead when the GPU does not start, but fails with a GPU that fails once started.
  pub fn compile(request: AiMapRequest<'_>, job: &JobHandle) -> XrfResult<AiMapCompilation> {
    let mut vertices: Vec<AiVertex> = Self::load(request.aimap)?;
    let gpu: Option<String> = match request.covers {
      // xrAI's draft leaves the covers as the allocator gave them, which on its fresh heap is zero.
      AiMapCovers::None => None,
      AiMapCovers::Full => {
        let rays = request.rays.ok_or_else(|| {
          XrfError::new_invalid_error("Full AI-map covers are traced against build.cform, and none was given")
        })?;

        AiCoverPass::compute(
          &mut vertices,
          request.aimap.params.patch_size,
          &AiCoverTracing {
            rays,
            backend: request.backend,
            dxc: request.dxc,
            gpu: request.gpu,
          },
          job,
        )?
      }
    };

    Ok(AiMapCompilation {
      gpu,
      ..Self::save(request.aimap.params.patch_size, &vertices, request.replaced)?
    })
  }
}
