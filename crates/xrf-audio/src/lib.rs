#![doc = include_str!("../README.md")]

pub(crate) mod acoustics;
pub(crate) mod control;
pub(crate) mod decode;
pub(crate) mod device;
pub(crate) mod host;
pub(crate) mod mixer;
pub(crate) mod offline;
pub(crate) mod render;
pub(crate) mod scene;
#[cfg(test)]
mod tests;
pub(crate) mod transport;
pub(crate) mod voice;

pub use crate::acoustics::audio_acoustics::AudioAcoustics;
pub use crate::acoustics::audio_spatialization::AudioSpatialization;
pub use crate::control::audio_engine::AudioEngine;
pub use crate::control::audio_engine_parts::AudioEngineParts;
pub use crate::control::audio_engine_settings::AudioEngineSettings;
pub use crate::control::audio_play::AudioPlay;
pub use crate::control::audio_refusal::AudioRefusal;
pub use crate::control::server::audio_event::AudioEvent;
pub use crate::control::server::audio_event_sink::AudioEventSink;
pub use crate::control::server::audio_gains::AudioGains;
pub use crate::control::server::audio_server::AudioServer;
pub use crate::decode::decoded_sound::DecodedSound;
pub use crate::decode::sound_channels::SoundChannels;
pub use crate::decode::sound_decoder::SoundDecoder;
pub use crate::decode::sound_format::SoundFormat;
pub use crate::decode::sound_peaks::SoundPeaks;
pub use crate::decode::stream::sound_stream::SoundStream;
pub use crate::decode::stream::sound_stream_start::SoundStreamStart;
pub use crate::decode::stream::stream_control::StreamControl;
pub use crate::decode::stream::stream_pump::StreamPump;
pub use crate::decode::stream::stream_pump_state::StreamPumpState;
pub use crate::decode::stream::stream_reader::StreamReader;
pub use crate::device::audio_device_description::AudioDeviceDescription;
pub use crate::device::audio_devices::AudioDevices;
pub use crate::device::audio_output::AudioOutput;
pub use crate::device::audio_output_choice::AudioOutputChoice;
pub use crate::device::audio_output_status::AudioOutputStatus;
pub use crate::host::audio_asset_source::AudioAssetSource;
pub use crate::host::audio_material::AudioMaterial;
pub use crate::host::audio_scene_geometry::AudioSceneGeometry;
pub use crate::host::audio_scene_mesh::AudioSceneMesh;
pub use crate::mixer::audio_mixer::AudioMixer;
pub use crate::mixer::audio_mixer_load::AudioMixerLoad;
pub use crate::mixer::audio_mixer_meter::AudioMixerMeter;
pub use crate::mixer::clock::audio_clock_anchor::AudioClockAnchor;
pub use crate::mixer::tap::audio_tap::AudioTap;
pub use crate::mixer::tap::audio_tap_block::AudioTapBlock;
pub use crate::mixer::tap::audio_tap_end::AudioTapEnd;
pub use crate::offline::offline_render::OfflineRender;
pub use crate::offline::offline_session::OfflineSession;
pub use crate::scene::audio_acoustics_report::AudioAcousticsReport;
pub use crate::scene::audio_emitter_report::AudioEmitterReport;
pub use crate::scene::audio_scene_refusal::AudioSceneRefusal;
pub use crate::scene::audio_scene_report::AudioSceneReport;
pub use crate::transport::audio_transport::AudioTransport;
pub use crate::transport::audio_transport_command::AudioTransportCommand;
pub use crate::transport::audio_transport_id::AudioTransportId;
pub use crate::transport::audio_transport_playback::AudioTransportPlayback;
pub use crate::transport::audio_transport_state::AudioTransportState;
pub use crate::voice::audio_voice_id::AudioVoiceId;
pub use crate::voice::audio_voice_playback::AudioVoicePlayback;
pub use crate::voice::audio_voice_state::AudioVoiceState;
pub use crate::voice::voice_source::VoiceSource;
// What a host tells the audio of a scene, so a host playing one names it here beside the server it plays through.
pub use xrf_audio_scene::{
  AudioBus, AudioListener, AudioSceneEmitter, AudioSceneFrame, AudioSceneInbox, AudioSceneRules,
};
