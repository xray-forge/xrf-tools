use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::viewer::LevelViewerFolder;
use crate::viewer::map::annotations::{AnnotationCanvas, LevelAnnotations, MapAnnotationsWriteRequest};

/// Draw the open level's annotations over one of its level maps and write them beside it as an SVG; answers the SVG.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "write_map_annotations")]
pub async fn levels_write_map_annotations(
  session_id: SessionId,
  request: MapAnnotationsWriteRequest,
  levels: State<'_, SelectedLevels>,
  directories: State<'_, AppDirectories>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<String>> {
  LevelViewerFolder::require_screenshot(&directories, &request.path)?;
  request.area.require_ground()?;

  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Drawing a level map's annotations",
    move |current, search| {
      AnnotationCanvas::over_map(&request)?.write(
        &LevelAnnotations::read(current, search)?,
        &LevelViewerFolder::get_annotations_path(&request.path),
      )
    },
  )
  .await
}
