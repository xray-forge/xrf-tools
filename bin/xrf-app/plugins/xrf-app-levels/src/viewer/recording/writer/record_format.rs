use std::time::Duration;

use crate::viewer::recording::writer::RecordSoundFormat;

/// A recording's format: the frame size, which H.264 needs even; the frame interval the file declares as its rate,
/// though each frame's own time and duration rule; and its sound, if any.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RecordFormat {
  pub width: u32,
  pub height: u32,
  pub frame: Duration,
  pub sound: Option<RecordSoundFormat>,
}
