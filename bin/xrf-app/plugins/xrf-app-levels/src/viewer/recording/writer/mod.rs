//! Video written with its sound into one file through Media Foundation's Sink Writer, on Windows only, which a recording
//! and a burst's video write through alike.

pub(crate) mod media;
#[cfg(windows)]
mod record_format;
#[cfg(windows)]
mod record_resampler;
#[cfg(windows)]
mod record_sample;
#[cfg(windows)]
mod record_sound_format;
#[cfg(windows)]
mod record_summary;
#[cfg(windows)]
mod record_writer;

pub use media::MediaProbe;
#[cfg(windows)]
pub use media::{MediaSession, MediaSink};
#[cfg(windows)]
pub use record_format::RecordFormat;
#[cfg(windows)]
pub use record_resampler::RecordResampler;
#[cfg(windows)]
pub use record_sample::RecordSample;
#[cfg(windows)]
pub use record_sound_format::RecordSoundFormat;
#[cfg(windows)]
pub use record_summary::RecordSummary;
#[cfg(windows)]
pub use record_writer::RecordWriter;
