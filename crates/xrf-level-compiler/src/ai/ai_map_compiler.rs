use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;

use crate::ai::ai_map_compilation::AiMapCompilation;
use crate::ai::ai_map_covers::AiMapCovers;
use crate::ai::ai_map_request::AiMapRequest;
use crate::ai::ai_vertex::AiVertex;
use crate::ai::cover::ai_cover_pass::AiCoverPass;
use crate::ai::cover::ai_cover_tracing::AiCoverTracing;

/// Compiles a level's AI map, `build.aimap` into `level.ai`, as SDK 0.7's `xrAI -f` does (`xrCompiler`,
/// `xrAI/compiler.cpp`): the nodes loaded, their covers computed, and the map saved sorted by cell. Node lighting is
/// not computed: xrAI's is dead code, and no loader reads the value.
pub(crate) struct AiMapCompiler;

impl AiMapCompiler {
  /// Compiles the map `request` names, its covers computed on the installed pool and the request's ray backend.
  ///
  /// # Errors
  ///
  /// Returns an error when `build.aimap` holds no node, is not the version xrAI loads or has a node the file's layout
  /// cannot hold, when full covers have no ray model, when the job is cancelled ([`XrfError::Cancelled`]), or when the
  /// GPU backend cannot trace or fails. `Auto` falls back to the CPU only where the GPU does not start.
  pub(crate) fn compile(request: AiMapRequest<'_>, job: &JobHandle) -> XrfResult<AiMapCompilation> {
    let mut vertices: Vec<AiVertex> = Self::load(request.aimap)?;
    let gpu: Option<String> = match request.covers {
      // xrAI's draft leaves the covers as the allocator gave them, which on its fresh heap is zero.
      AiMapCovers::None => None,
      AiMapCovers::Full => {
        let scene = request.scene.ok_or_else(|| {
          XrfError::new_invalid_error("Full AI-map covers are traced against build.cform, and none was given")
        })?;

        AiCoverPass::compute(
          &mut vertices,
          request.aimap.params.patch_size,
          &AiCoverTracing {
            scene,
            tracing: request.tracing,
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
