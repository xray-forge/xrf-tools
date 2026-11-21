use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;

use crate::edit::{TextureSaveAsPlan, TexturesPlanSaveAsRequest};

/// Say what a Save as under a given name would write, without writing; the dialog shows this plan.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "plan_save_as")]
pub(crate) async fn textures_plan_save_as(
  request: TexturesPlanSaveAsRequest,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TextureSaveAsPlan> {
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Planning a texture Save as", move || {
      TextureSaveAsPlan::build(
        &assets.search(&request.target.roots)?,
        &request.target,
        &request.form,
        request.bump,
      )
    })
    .await?
}
