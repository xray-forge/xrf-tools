use std::sync::mpsc::SyncSender;

use xrf_error::XrfResult;

use crate::control::server::audio_gains::AudioGains;
use crate::transport::audio_transport::AudioTransport;
use crate::transport::audio_transport_command::AudioTransportCommand;
use crate::transport::audio_transport_id::AudioTransportId;
use crate::transport::audio_transport_state::AudioTransportState;

/// What an [`crate::AudioServer`] sends its control thread; a request with an `answer` is waited on.
pub(crate) enum ControlRequest {
  SetGains(AudioGains),
  OpenTransport {
    id: AudioTransportId,
    transport: Box<AudioTransport>,
  },
  SteerTransport {
    id: AudioTransportId,
    command: AudioTransportCommand,
    answer: SyncSender<XrfResult<()>>,
  },
  DescribeTransport {
    id: AudioTransportId,
    answer: SyncSender<Option<AudioTransportState>>,
  },
  /// Sends the transport's state again on the next tick, whether or not it changed.
  ResendTransport(AudioTransportId),
  CloseTransport(AudioTransportId),
  /// Closes every transport and ends the thread.
  ShutDown,
}
