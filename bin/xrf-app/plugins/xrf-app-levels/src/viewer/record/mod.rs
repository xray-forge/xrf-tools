//! Video written with its sound into one file through Media Foundation's Sink Writer, on Windows only: its libraries
//! are delay-loaded, so the probe answers whether this Windows has them before anything binds to them.

mod media_probe;
#[cfg(windows)]
mod media_session;
#[cfg(windows)]
mod media_sink;
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

pub use media_probe::MediaProbe;
#[cfg(windows)]
pub use media_session::MediaSession;
#[cfg(windows)]
pub use media_sink::MediaSink;
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
