use std::time::Duration;

use crate::viewer::recording::writer::RecordSoundFormat;

/// A recording's picture and sound: its frames' size, which the H.264 encoder takes only even; the time between its
/// frames as the file names its frame rate, which each frame's own time and duration override; and its sound's, where it
/// has one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RecordFormat {
  pub width: u32,
  pub height: u32,
  pub frame: Duration,
  pub sound: Option<RecordSoundFormat>,
}
