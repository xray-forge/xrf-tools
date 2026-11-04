use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use xrf_error::XrfResult;

use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::control::server::audio_gains::AudioGains;
use crate::host::audio_asset_source::AudioAssetSource;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::scene::audio_scene_inbox::SceneMailbox;
use crate::scene::audio_scene_report::AudioSceneReport;
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
  /// Plays a scene whose frames come through `mailbox`, its sounds read from `assets`, shaped by `geometry` where the
  /// host has it.
  OpenScene {
    mailbox: SceneMailbox,
    assets: Arc<dyn AudioAssetSource>,
    geometry: Option<Arc<dyn AudioSceneGeometry>>,
  },
  /// Places sounds for headphones or speakers from the next tick.
  SetSpatialization(AudioSpatialization),
  /// How the scene the host names `scene` plays, none for a scene not open.
  DescribeScene {
    scene: u32,
    answer: SyncSender<Option<AudioSceneReport>>,
  },
  /// Closes every transport and scene and ends the thread.
  ShutDown,
}
