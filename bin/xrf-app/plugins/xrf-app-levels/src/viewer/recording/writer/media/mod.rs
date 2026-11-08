//! Media Foundation itself: its libraries are delay-loaded, so the probe answers whether this Windows has them before
//! anything binds to them; the session holds it started on a thread, and the sink is its Sink Writer.

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
