use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_audio::SoundPeaks;

use xrf_audio::AudioTransportId;

use crate::preview::PreviewTransports;

/// The preview's waveform: the loudest sample of each of `count` equal stretches of its sound, decoded on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "read_preview_peaks")]
pub(crate) async fn audio_read_preview_peaks(
  execution: State<'_, ExecutionState>,
  transports: State<'_, PreviewTransports>,
  transport: AudioTransportId,
  count: u32,
) -> TauriResult<SoundPeaks> {
  let transports: PreviewTransports = transports.inner().clone();

  execution
    .run_blocking("Reading a preview's waveform", move || {
      transports.read_peaks(transport, count as usize)
    })
    .await?
}
