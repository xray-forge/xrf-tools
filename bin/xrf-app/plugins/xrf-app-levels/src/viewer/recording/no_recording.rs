use xrf_app_core::TauriResult;

use crate::viewer::recording::{RecordingOutcome, SlotRecording};

/// No recording at all: nothing records off Windows, so the slot stays empty there.
pub(crate) enum NoRecording {}

impl SlotRecording for NoRecording {
  fn get_window(&self) -> &str {
    match *self {}
  }

  fn is_ended(&self) -> bool {
    match *self {}
  }

  fn stop(self) -> TauriResult<RecordingOutcome> {
    match self {}
  }
}
