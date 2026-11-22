use crate::viewer::recording::RecordingSlot;

/// The level viewer's recording slot, of the recorder this platform has: Media Foundation's on Windows.
#[cfg(windows)]
pub(crate) type LevelRecordings = RecordingSlot<crate::viewer::recording::ViewportRecorder>;

/// The level viewer's recording slot, which nothing fills off Windows.
#[cfg(not(windows))]
pub(crate) type LevelRecordings = RecordingSlot<crate::viewer::recording::NoRecording>;
