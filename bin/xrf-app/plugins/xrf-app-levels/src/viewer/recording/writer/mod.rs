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

pub(crate) use media::MediaProbe;
#[cfg(windows)]
pub(crate) use media::{MediaSession, MediaSink};
#[cfg(windows)]
pub(crate) use record_format::RecordFormat;
#[cfg(windows)]
pub(crate) use record_resampler::RecordResampler;
#[cfg(windows)]
pub(crate) use record_sample::RecordSample;
#[cfg(windows)]
pub(crate) use record_sound_format::RecordSoundFormat;
#[cfg(windows)]
pub(crate) use record_summary::RecordSummary;
#[cfg(windows)]
pub(crate) use record_writer::RecordWriter;
