use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_dds::ImageFormat;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{PackEquipmentOptions, PackEquipmentProcessor, PackEquipmentResult};

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::lease::{PACK_SPRITE_JOB_KIND, to_pack_sprite_lease_key};

/// Draw every declared inventory icon into one equipment sprite sheet.
///
/// Holds the sheet it writes exclusively, so a second request for the same output is refused rather than allowed to
/// race it. A cancelled run leaves nothing behind: the sheet is one image written once at the end, so stopping before
/// that point writes no file at all.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_sprite"))]
#[tauri::command(rename = "pack_sprite")]
pub async fn sprite_equipment_pack_sprite(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  source_path: &str,
  output_path: &str,
  system_ltx_path: &str,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<PackEquipmentResult> {
  log::info!("Packing equipment dds: {source_path} -> {output_path}, {system_ltx_path}");

  let source: PathBuf = source_path.into();
  let output: PathBuf = output_path.into();
  let system_ltx: PathBuf = system_ltx_path.into();

  // Registered before the hop, and before the LTX is read: `system.ltx` pulls in the whole include tree, which on an
  // installation is thousands of files and most of the wait.
  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, PACK_SPRITE_JOB_KIND)
      .with_lease_keys(vec![to_pack_sprite_lease_key(&output)])
      .with_request(&json!({ "source": source, "output": output, "systemLtx": system_ltx }))
      .with_progress(progress),
  )?;

  // Off the async worker: reading the include tree, decoding every icon, and encoding one sheet is not work an IPC
  // executor should be holding. It was on that executor until this became a job.
  let packing: JobHandle = job.clone();
  let outcome: TauriResult<PackEquipmentResult> = execution
    .run_blocking("Equipment sprite pack", move || {
      let options: PackEquipmentOptions = PackEquipmentOptions {
        job: packing,
        ltx: Ltx::read_from_file_full(&system_ltx)?,
        source,
        output: OutputOptions::default(),
        output_path: output,
        gamedata: None,
        dds_compression_format: ImageFormat::BC3RgbaUnorm,
        is_strict: false,
      };

      PackEquipmentProcessor::pack_sprites(options)
    })
    .await?
    .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
