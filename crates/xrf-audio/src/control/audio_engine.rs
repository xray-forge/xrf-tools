use std::sync::Arc;

use rtrb::{Consumer, Producer};

use crate::control::audio_play::AudioPlay;
use crate::control::audio_refusal::AudioRefusal;
use crate::control::voice_slots::VoiceSlots;
use crate::mixer::audio_bus::AudioBus;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;
use crate::mixer::mixer_command::MixerCommand;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::audio_voice_playback::AudioVoicePlayback;
use crate::voice::audio_voice_state::AudioVoiceState;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::shared::voice_status::VoiceStatus;
use crate::voice::voice::Voice;

/// The control side of the audio: it starts, steers and stops voices on the mixer it was made with
/// ([`crate::AudioEngineParts::new`]), from any one thread, without waiting on the audio thread.
///
/// A voice's life (start, pause, resume, stop) and the mix's levels travel as commands, which a full ring refuses. A
/// voice's continuous controls (gain, rate) are written into its slot's parameter block, which never refuses and keeps
/// only the latest write, however many a buffer brings.
///
/// Voices the mixer is done with come back here to be freed, the next time a voice starts or [`Self::collect`] runs.
pub struct AudioEngine {
  commands: Producer<MixerCommand>,
  retired: Consumer<Box<Voice>>,
  parameters: Arc<[VoiceParameters]>,
  statuses: Arc<[VoiceStatus]>,
  slots: VoiceSlots,
  /// Each slot's voice's first frame, reported while the audio thread has not taken the voice up.
  start_frames: Box<[u64]>,
  meter: AudioMixerMeter,
}

impl AudioEngine {
  pub(crate) fn new(
    commands: Producer<MixerCommand>,
    retired: Consumer<Box<Voice>>,
    parameters: Arc<[VoiceParameters]>,
    statuses: Arc<[VoiceStatus]>,
    meter: AudioMixerMeter,
  ) -> Self {
    let slots: VoiceSlots = VoiceSlots::new(statuses.len());
    let start_frames: Box<[u64]> = vec![0; statuses.len()].into_boxed_slice();

    Self {
      commands,
      retired,
      parameters,
      statuses,
      slots,
      start_frames,
      meter,
    }
  }

  /// The mixer's clock and load.
  pub fn get_meter(&self) -> &AudioMixerMeter {
    &self.meter
  }

  /// Starts a voice, which the audio thread takes up with the next buffer.
  ///
  /// # Errors
  ///
  /// Refuses the voice when every slot plays or the audio thread has not taken its commands; the source is dropped.
  pub fn play(&mut self, play: AudioPlay) -> Result<AudioVoiceId, AudioRefusal> {
    self.collect();

    let id: AudioVoiceId = self.slots.take().ok_or(AudioRefusal::VoicesExhausted)?;
    let parameters: &VoiceParameters = &self.parameters[id.slot as usize];

    // The slot is free, so no voice reads its block until this one starts with what is written here.
    parameters.set_gain(play.gain);
    parameters.set_rate(play.rate);
    self.start_frames[id.slot as usize] = play.source.get_start_frame();

    let voice: Box<Voice> = Box::new(Voice::new(id, play.source, play.bus));

    if self.commands.push(MixerCommand::Start(voice)).is_err() {
      self.slots.give_back(id);
      return Err(AudioRefusal::CommandsFull);
    }

    Ok(id)
  }

  /// Fades the voice out and ends it. A voice that has ended already is left alone.
  ///
  /// # Errors
  ///
  /// Refuses when the audio thread has not taken its commands.
  pub fn stop(&mut self, voice: AudioVoiceId) -> Result<(), AudioRefusal> {
    self.send(MixerCommand::Stop(voice))
  }

  /// Fades the voice out and holds its place.
  ///
  /// # Errors
  ///
  /// Refuses when the audio thread has not taken its commands.
  pub fn pause(&mut self, voice: AudioVoiceId) -> Result<(), AudioRefusal> {
    self.send(MixerCommand::Pause(voice))
  }

  /// Fades a paused voice back in where it was.
  ///
  /// # Errors
  ///
  /// Refuses when the audio thread has not taken its commands.
  pub fn resume(&mut self, voice: AudioVoiceId) -> Result<(), AudioRefusal> {
    self.send(MixerCommand::Resume(voice))
  }

  /// Moves the voice's linear gain to `gain` from the next buffer; below zero or not a number is silence. A voice
  /// that has ended is left alone.
  pub fn set_voice_gain(&mut self, voice: AudioVoiceId, gain: f32) {
    if let Some(parameters) = self.get_parameters(voice) {
      parameters.set_gain(gain);
    }
  }

  /// Sets the voice's playback speed from the next buffer, 1 at the source's own rate; not a number above zero plays
  /// at 1. A voice that has ended is left alone.
  pub fn set_voice_rate(&mut self, voice: AudioVoiceId, rate: f32) {
    if let Some(parameters) = self.get_parameters(voice) {
      parameters.set_rate(rate);
    }
  }

  /// Moves the bus's linear gain to `gain`.
  ///
  /// # Errors
  ///
  /// Refuses when the audio thread has not taken its commands.
  pub fn set_bus_gain(&mut self, bus: AudioBus, gain: f32) -> Result<(), AudioRefusal> {
    self.send(MixerCommand::SetBusGain { bus, gain })
  }

  /// Moves the linear gain of everything played to `gain`.
  ///
  /// # Errors
  ///
  /// Refuses when the audio thread has not taken its commands.
  pub fn set_master_gain(&mut self, gain: f32) -> Result<(), AudioRefusal> {
    self.send(MixerCommand::SetMasterGain(gain))
  }

  /// Where the voice is. A voice whose slot has been freed has ended, whatever its last report said.
  pub fn get_playback(&self, voice: AudioVoiceId) -> AudioVoicePlayback {
    let status: Option<AudioVoicePlayback> = self
      .statuses
      .get(voice.slot as usize)
      .and_then(|status| status.read(voice.generation));

    match status {
      Some(playback) => playback,
      None if self.slots.is_current(voice) => AudioVoicePlayback {
        state: AudioVoiceState::Starting,
        frame: self.start_frames[voice.slot as usize],
      },
      None => AudioVoicePlayback {
        state: AudioVoiceState::Ended,
        frame: 0,
      },
    }
  }

  /// Voices playing or waiting to start, of [`Self::get_voices_count`].
  pub fn get_playing_count(&self) -> usize {
    self.slots.get_playing_count()
  }

  pub fn get_voices_count(&self) -> usize {
    self.slots.get_count()
  }

  /// Frees the voices the mixer is done with, and their slots.
  pub fn collect(&mut self) {
    while let Ok(voice) = self.retired.pop() {
      self.slots.give_back(voice.id);
    }
  }

  /// The parameter block of `voice`, while its slot still holds it: a block is never written for a voice whose slot
  /// another has taken.
  fn get_parameters(&mut self, voice: AudioVoiceId) -> Option<&VoiceParameters> {
    self.collect();

    self
      .slots
      .is_current(voice)
      .then(|| &self.parameters[voice.slot as usize])
  }

  fn send(&mut self, command: MixerCommand) -> Result<(), AudioRefusal> {
    self.collect();
    self.commands.push(command).map_err(|_| AudioRefusal::CommandsFull)
  }
}
