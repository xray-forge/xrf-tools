use crate::viewer::recording::RecordingAvailability;
use crate::viewer::recording::writer::MediaProbe;

/// Whether a viewport can be recorded here: Media Foundation loads; why not where not.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_recording")]
pub(crate) fn levels_describe_recording() -> RecordingAvailability {
  let reason: Option<String> = MediaProbe::MEDIA_FOUNDATION.load().err();

  RecordingAvailability {
    is_available: reason.is_none(),
    reason,
  }
}
