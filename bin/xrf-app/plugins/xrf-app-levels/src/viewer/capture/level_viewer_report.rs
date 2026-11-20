use std::path::Path;

use xrf_renderer::{RenderCubeCapture, RenderMapCapture, RenderViewportId};

/// What the level viewer says to the log, in one place rather than beside each command that says it.
pub(crate) struct LevelViewerReport;

impl LevelViewerReport {
  /// That a level map capture has begun.
  pub(crate) fn capturing_map(capture: &RenderMapCapture, viewport: RenderViewportId, path: &Path) {
    log::info!(
      "Capturing a {} by {} level map in viewport {viewport} to '{}'",
      capture.width,
      capture.height,
      path.display()
    );
  }

  /// That a cubemap capture has begun.
  pub(crate) fn capturing_cube(capture: &RenderCubeCapture, viewport: RenderViewportId, path: &Path) {
    log::info!(
      "Capturing a cubemap of {} pixel faces in viewport {viewport} to '{}'",
      capture.size,
      path.display()
    );
  }

  /// That a level map is exported.
  pub(crate) fn exporting_map(source: &Path, target: &Path) {
    log::info!("Exporting level map '{}' as '{}'", source.display(), target.display());
  }

  /// That a cubemap is exported.
  pub(crate) fn exporting_cube(source: &Path, target: &Path) {
    log::info!("Exporting cubemap '{}' as '{}'", source.display(), target.display());
  }
}
