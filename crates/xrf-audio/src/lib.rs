#![doc = include_str!("../README.md")]

pub(crate) mod control;
pub(crate) mod decode;
pub(crate) mod device;
pub(crate) mod mixer;
pub(crate) mod offline;
#[cfg(test)]
mod tests;
pub(crate) mod transport;
pub(crate) mod voice;

pub use crate::control::audio_engine::AudioEngine;
pub use crate::control::audio_engine_parts::AudioEngineParts;
pub use crate::control::audio_engine_settings::AudioEngineSettings;
pub use crate::control::audio_play::AudioPlay;
pub use crate::control::audio_refusal::AudioRefusal;
pub use crate::decode::decoded_sound::DecodedSound;
pub use crate::decode::sound_channels::SoundChannels;
pub use crate::decode::sound_decoder::SoundDecoder;
pub use crate::decode::sound_peaks::SoundPeaks;
pub use crate::decode::stream::sound_stream::SoundStream;
pub use crate::decode::stream::sound_stream_start::SoundStreamStart;
pub use crate::decode::stream::stream_control::StreamControl;
pub use crate::decode::stream::stream_reader::StreamReader;
pub use crate::device::audio_device_description::AudioDeviceDescription;
pub use crate::device::audio_devices::AudioDevices;
pub use crate::device::audio_output::AudioOutput;
pub use crate::device::audio_output_choice::AudioOutputChoice;
pub use crate::device::audio_output_status::AudioOutputStatus;
pub use crate::mixer::audio_bus::AudioBus;
pub use crate::mixer::audio_mixer::AudioMixer;
pub use crate::offline::offline_render::OfflineRender;
pub use crate::transport::audio_transport::AudioTransport;
pub use crate::transport::audio_transport_playback::AudioTransportPlayback;
pub use crate::transport::audio_transport_state::AudioTransportState;
pub use crate::voice::audio_voice_id::AudioVoiceId;
pub use crate::voice::audio_voice_playback::AudioVoicePlayback;
pub use crate::voice::audio_voice_state::AudioVoiceState;
pub use crate::voice::voice_source::VoiceSource;
