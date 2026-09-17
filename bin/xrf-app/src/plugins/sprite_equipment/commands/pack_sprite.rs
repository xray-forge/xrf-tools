use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_dds::ImageFormat;
use xrf_dltx::select_ltx_dialect;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{PackEquipmentOptions, PackEquipmentProcessor, PackEquipmentResult};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::request::PackSpriteRequest;

/// Draw every declared inventory icon into one equipment sprite sheet.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_sprite"))]
#[tauri::command(rename = "pack_sprite")]
pub async fn sprite_equipment_pack_sprite(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: PackSpriteRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<PackEquipmentResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::SpriteEquipmentPack).with_request(&request);

  let PackSpriteRequest {
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

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::SpriteEquipmentPack.as_str())
      .with_resources(vec![JobResource::file(&output)])
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Equipment sprite pack",
    registration,
    move || {
      let options: PackEquipmentOptions = PackEquipmentOptions {
        job,
        ltx: Ltx::read_from_file_with_dialect(&system_ltx, select_ltx_dialect(is_dltx).as_ref())?,
        source,
        output: OutputOptions::default(),
        output_path: output,
        gamedata: None,
        dds_compression_format: ImageFormat::BC3RgbaUnorm,
        is_strict: false,
      };

      PackEquipmentProcessor::pack_sprites(options)
    },
    |result| result.outcome,
  )
  .await
}
