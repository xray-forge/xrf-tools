//! A viewport recorded with what its camera hears into one video, through the renderer's recording, the audio's tap
//! and its clock, and Media Foundation's writer.

mod level_recordings;
mod recording_availability;
mod recording_outcome;
mod recording_progress;
#[cfg(windows)]
mod viewport_recorder;
pub(crate) mod writer;

pub use level_recordings::LevelRecordings;
pub use recording_availability::RecordingAvailability;
pub use recording_outcome::RecordingOutcome;
pub use recording_progress::RecordingProgress;
#[cfg(windows)]
pub use viewport_recorder::ViewportRecorder;
