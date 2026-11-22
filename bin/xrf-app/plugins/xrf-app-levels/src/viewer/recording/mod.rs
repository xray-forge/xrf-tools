//! Recording a viewport and what its camera hears into one video, through the renderer's recording, the audio tap and
//! clock, and Media Foundation's writer.

mod level_recordings;
#[cfg(not(windows))]
mod no_recording;
mod recording_availability;
mod recording_outcome;
mod recording_progress;
mod recording_slot;
mod slot_recording;
#[cfg(windows)]
mod viewport_recorder;
pub(crate) mod writer;

pub(crate) use level_recordings::LevelRecordings;
#[cfg(not(windows))]
pub(crate) use no_recording::NoRecording;
pub(crate) use recording_availability::RecordingAvailability;
pub(crate) use recording_outcome::RecordingOutcome;
pub(crate) use recording_progress::RecordingProgress;
pub(crate) use recording_slot::RecordingSlot;
pub(crate) use slot_recording::SlotRecording;
#[cfg(windows)]
pub(crate) use viewport_recorder::ViewportRecorder;
