use std::time::Duration;

/// What a caller asks of an open transport, as [`crate::AudioTransport`]'s methods do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AudioTransportCommand {
  Play,
  Pause,
  Stop,
  Seek(Duration),
  SetLooped(bool),
}
