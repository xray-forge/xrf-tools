use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_dds::ImageFormat;
use xrf_dltx::DltxDialect;
use xrf_job::JobProgress;
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{EquipmentSpritePackOptions, EquipmentSpritePackResult, EquipmentSpritePacker};

use crate::ipc::sprite_equipment::SPRITE_EQUIPMENT_JOB_PACK;
use crate::request::SpriteEquipmentPackRequest;

/// Draw every declared inventory icon into one equipment sprite sheet.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_sprite"))]
#[tauri::command(rename = "pack_sprite")]
pub async fn sprite_equipment_pack_sprite(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: SpriteEquipmentPackRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<EquipmentSpritePackResult> {
  let start: JobStart = JobStart::new(job_id, SPRITE_EQUIPMENT_JOB_PACK)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::file(&request.output_path)])
    .with_progress(progress);

  let SpriteEquipmentPackRequest {
    source_path: source,
    output_path: output,
    system_ltx_path: system_ltx,
    is_dltx,
  } = request;

  log::info!(
    "Packing equipment dds: {} -> {}, {}",
    source.display(),
    output.display(),
    system_ltx.display()
  );

  registry
    .run(
      &execution,
      start,
      move |job| {
        EquipmentSpritePacker::pack_sprites(EquipmentSpritePackOptions {
          job,
          ltx: Ltx::read_from_file_with_dialect(&system_ltx, DltxDialect::select(is_dltx).as_ref())?,
          source,
          output: OutputOptions::default(),
          output_path: output,
          gamedata: None,
          dds_compression_format: ImageFormat::BC3RgbaUnorm,
          is_strict: false,
        })
      },
      |result| result.outcome,
    )
    .await
}
