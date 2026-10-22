use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_utils::write_file_staged;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::spawn::{get_level_graph, get_level_spawn};
use crate::plugins::levels::state::{LevelState, SelectedLevel};
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::levels::viewer::annotations::annotation_svg::AnnotationCanvas;
use crate::plugins::levels::viewer::annotations::level_annotations::LevelAnnotations;
use crate::plugins::levels::viewer::annotations::map_annotations_request::MapAnnotationsRequest;
use crate::plugins::system::paths::SystemPathsState;

/// Draw the open level's annotations over one of its level maps and write them beside it as an SVG; answers the SVG.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "write_map_annotations"))]
#[tauri::command(rename = "write_map_annotations")]
pub async fn levels_write_map_annotations(
  session_id: SessionId,
  request: MapAnnotationsRequest,
  state: State<'_, LevelState>,
  paths: State<'_, SystemPathsState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<String>> {
  LevelViewerFolder::require_screenshot(paths.local_data.clone(), &request.path)?;
  request.area.require_ground()?;

  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: String = execution
    .run_blocking("Drawing a level map's annotations", move || {
      assets.with_probe(&current.roots, |probe| -> TauriResult<String> {
        let level: String = current
          .source
          .get_name()
          .ok_or_else(|| format!("Level {} has no name to find its spawn by", current.source.get_label()))?;
        let annotations: LevelAnnotations = LevelAnnotations::of(
          &*get_level_spawn(&current, probe)?,
          &*get_level_graph(&current, probe)?,
          &level,
        );
        let (width, height): (u32, u32) = image::image_dimensions(&request.path)
          .map_err(|error| format!("Failed to read map '{}': {error}", request.path.display()))?;
        let svg: String = AnnotationCanvas {
          area: request.area,
          width,
          height,
          kinds: request.kinds,
          is_labelled: request.is_labelled,
        }
        .draw(&annotations);
        let target: PathBuf = LevelViewerFolder::get_annotations_path(&request.path);

        write_file_staged(&target, svg.as_bytes())
          .map_err(|error| format!("Failed to write annotations '{}': {error}", target.display()))?;

        Ok(svg)
      })
    })
    .await???;

  Ok(SessionSnapshot::new(session_id, value))
}
