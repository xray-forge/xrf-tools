//! Media Foundation bindings: the probe checks the delay-loaded libraries exist before anything binds to them, the
//! session keeps it started on a thread, and the sink wraps its Sink Writer.

mod media_probe;
#[cfg(windows)]
mod media_session;
#[cfg(windows)]
mod media_sink;

pub(crate) use media_probe::MediaProbe;
#[cfg(windows)]
pub(crate) use media_session::MediaSession;
#[cfg(windows)]
pub(crate) use media_sink::MediaSink;
