use crate::transport::audio_transport_id::AudioTransportId;
use crate::transport::audio_transport_state::AudioTransportState;

/// What the control thread tells its host, as it happens.
#[derive(Clone, Debug, PartialEq)]
pub enum AudioEvent {
  /// The transport's state changed: its playback, looping or failure at once, its position while it plays at most
  /// every [`crate::AudioServer::POSITION_INTERVAL`].
  TransportChanged {
    id: AudioTransportId,
    state: AudioTransportState,
  },
}
