use std::time::Instant;

use xrf_app_core::session::SessionSnapshot;
use xrf_renderer::RenderViewportId;

use crate::scene::{SceneSource, SelectedScene};

/// What opening and showing a scene says to the log, in one place rather than beside each command that says it; a
/// level's lines are `LevelReport`'s.
pub(crate) struct SceneReport;

impl SceneReport {
  /// That a scene's open has begun.
  pub(crate) fn opening(source: &SceneSource) {
    log::info!("Opening scene: {}", source.path);
  }

  /// What a scene's open read, and each part file the editor would skip or refuse.
  pub(crate) fn opened(opened: &SessionSnapshot<SelectedScene>, started: Instant) {
    let scene: &SelectedScene = &opened.value;

    log::info!(
      "Opened scene {} in {}: {} tools, {} loaded, {} inputs read",
      scene.source.path,
      xrf_utils::format_duration(started.elapsed()),
      scene.tools.len(),
      scene.scene.tools.len(),
      opened.inputs.len()
    );

    for problem in &scene.problems {
      log::warn!(
        "Scene {} skips {} of tool '{}': {}",
        scene.source.path,
        problem.file.as_deref().unwrap_or("the objects"),
        problem.tool,
        problem.message
      );
    }
  }

  /// That a scene is shown in a viewport, and where its textures come from.
  pub(crate) fn showing(scene: &SelectedScene, viewport: RenderViewportId) {
    log::info!(
      "Showing scene {} in native viewport {viewport}, its textures from {}",
      scene.source.path,
      scene.game.roots.describe()
    );
  }

  /// That a scene's library object could not be read or compiled, so its placements draw nothing.
  pub(crate) fn undrawn_object(reference: &str, reason: &str) {
    log::warn!("Scene library object '{reference}' is not drawn: {reason}");
  }
}
