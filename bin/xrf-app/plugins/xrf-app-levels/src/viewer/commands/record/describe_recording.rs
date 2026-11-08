use tauri::State;

use crate::render::ShownLevels;
use crate::viewer::recording::RecordingAvailability;
use crate::viewer::recording::writer::MediaProbe;

/// Whether a viewport can be recorded here: Media Foundation loads and the audio started; why not where not.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_recording")]
pub(crate) fn levels_describe_recording(shown: State<'_, ShownLevels>) -> RecordingAvailability {
  let reason: Option<String> = MediaProbe::MEDIA_FOUNDATION.load().err().or_else(|| {
    shown
      .get_audio()
      .server
      .is_none()
      .then(|| String::from("The audio did not start, so a recording would have no sound"))
  });

  RecordingAvailability {
    is_available: reason.is_none(),
    reason,
  }
}
