use std::sync::Arc;

use xrf_error::XrfResult;

use crate::decode::sound_format::SoundFormat;
use crate::decode::stream::sound_stream::SoundStream;
use crate::decode::stream::sound_stream_start::SoundStreamStart;
use crate::decode::stream::stream_pump::{StreamPump, StreamPumpState};
use crate::decode::stream::stream_reader::StreamReader;

/// How the control thread starts a stream: each on a thread of its own, or pumped by whoever renders, as an offline
/// session does so that a render with streams comes out the same every time.
pub(crate) enum StreamLauncher {
  Threads,
  Pumped(Vec<StreamPump>),
}

impl StreamLauncher {
  pub(crate) fn start(
    &mut self,
    bytes: Arc<[u8]>,
    format: SoundFormat,
    start: SoundStreamStart,
  ) -> XrfResult<StreamReader> {
    match self {
      Self::Threads => SoundStream::start(bytes, format, start),
      Self::Pumped(pumps) => {
        let (reader, pump) = SoundStream::start_pumped(bytes, format, start);

        pumps.push(pump);

        Ok(reader)
      }
    }
  }

  /// Fills every pumped stream's ring, letting go of those that are done.
  pub(crate) fn pump(&mut self) {
    if let Self::Pumped(pumps) = self {
      pumps.retain_mut(|pump| pump.fill() == StreamPumpState::Full);
    }
  }
}
