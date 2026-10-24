use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::app_directories::AppDirectories;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_utils::write_file_staged;
use xrf_vfs::XraySearch;

use crate::parts::{LevelGraphPart, LevelSpawnPart};
use crate::state::{LevelState, SelectedLevel};
use crate::viewer::LevelViewerFolder;
use crate::viewer::annotations::annotation_svg::AnnotationCanvas;
use crate::viewer::annotations::level_annotations::LevelAnnotations;
use crate::viewer::annotations::map_annotations_request::MapAnnotationsRequest;

/// Draw the open level's annotations over one of its level maps and write them beside it as an SVG; answers the SVG.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "write_map_annotations"))]
#[tauri::command(rename = "write_map_annotations")]
pub async fn levels_write_map_annotations(
  session_id: SessionId,
  request: MapAnnotationsRequest,
  state: State<'_, LevelState>,
  directories: State<'_, AppDirectories>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<String>> {
  LevelViewerFolder::require_screenshot(&directories, &request.path)?;
  request.area.require_ground()?;

  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: SessionSnapshot<String> = execution
    .run_blocking("Drawing a level map's annotations", move || -> TauriResult<_> {
      let search: XraySearch = assets.search(&current.roots)?;
      let level: String = current
        .source
        .get_name()
        .ok_or_else(|| format!("Level {} has no name to find its spawn by", current.source.get_label()))?;
      let annotations: LevelAnnotations = LevelAnnotations::of(
        &*LevelSpawnPart::get(&current, &search)?,
        &*LevelGraphPart::get(&current, &search)?,
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

      Ok(current.answer(svg))
    })
    .await??;

  Ok(value)
}
