use xrf_app_core::TauriResult;

use crate::viewer::recording::RecordingOutcome;

/// A recording a [`RecordingSlot`](crate::viewer::recording::RecordingSlot) holds: the page that started it, whether it
/// ended by itself, and how it finishes. The slot's protocol reads nothing else of it, so it is tried with a stand-in.
pub(crate) trait SlotRecording: Send + 'static {
  /// The label of the window whose page started it; the recording ends with that page.
  fn get_window(&self) -> &str;

  /// Whether it ended by itself and only awaits collection.
  fn is_ended(&self) -> bool;

  /// Stops it and finishes its file, answering what was written.
  ///
  /// # Errors
  ///
  /// Returns an error when the file could not be finished.
  fn stop(self) -> TauriResult<RecordingOutcome>;
}
