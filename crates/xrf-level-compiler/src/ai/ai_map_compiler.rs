use xrf_error::{XrfError, XrfResult};

use crate::ai::ai_map_compilation::AiMapCompilation;
use crate::ai::ai_map_covers::AiMapCovers;
use crate::ai::ai_map_request::AiMapRequest;
use crate::ai::ai_vertex::AiVertex;

/// Compiles a level's AI map, `build.aimap` into `level.ai`, as SDK 0.7's `xrAI -f` does (`xrCompiler`,
/// `xrAI/compiler.cpp:63`): the nodes loaded, their covers computed, and the map saved sorted by cell. Node lighting is
/// not computed: xrAI's is dead code, and no loader reads the value.
pub struct AiMapCompiler;

impl AiMapCompiler {
  /// Compiles the map `request` names.
  ///
  /// # Errors
  ///
  /// Returns an error when `build.aimap` holds no node or is not the version xrAI loads, when a node does not fit the
  /// file's layout, or for covers not yet computed.
  pub fn compile(request: AiMapRequest<'_>) -> XrfResult<AiMapCompilation> {
    let vertices: Vec<AiVertex> = Self::load(request.aimap)?;

    match request.covers {
      // xrAI's draft leaves the covers as the allocator gave them, which on its fresh heap is zero.
      AiMapCovers::None => {}
      AiMapCovers::Full => {
        // todo: Compute covers from `build.cform`: trace the cover nodes and mix the others, as xrAI's `xrCover` does.
        return Err(XrfError::new_not_implemented_error(
          "The AI map compiler computes no covers yet; compile with covers none",
        ));
      }
    }

    Self::save(request.aimap.params.patch_size, &vertices, request.replaced)
  }
}
