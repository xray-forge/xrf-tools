use std::sync::Arc;
use std::time::Duration;

use rtrb::{Consumer, Producer};

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::control::audio_play::AudioPlay;
use crate::control::audio_refusal::AudioRefusal;
use crate::control::engine_shared::EngineShared;
use crate::control::voice_slots::VoiceSlots;
use crate::mixer::audio_bus::AudioBus;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;
use crate::mixer::mixer_command::MixerCommand;
use crate::mixer::reverb_parameters::ReverbParameters;
use crate::mixer::tap::audio_tap::AudioTap;
use crate::mixer::tap::mixer_tap::MixerTap;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::audio_voice_playback::AudioVoicePlayback;
use crate::voice::audio_voice_state::AudioVoiceState;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;
use crate::voice::shared::voice_status::VoiceStatus;
use crate::voice::voice::Voice;
use crate::voice::voice_spatial::VoiceSpatial;

/// The control side of the audio: it starts, steers and stops voices on the mixer it was made with
/// ([`crate::AudioEngineParts::new`]), from any one thread, without waiting on the audio thread.
pub struct AudioEngine {
  commands: Producer<MixerCommand>,
  retired: Consumer<Box<Voice>>,
  /// Taps the mixer let go of, freed here.
  retired_taps: Consumer<Box<MixerTap>>,
  parameters: Arc<[VoiceParameters]>,
  statuses: Arc<[VoiceStatus]>,
  slots: VoiceSlots,
  /// Each slot's voice's first frame, reported while the audio thread has not taken the voice up.
  start_frames: Box<[u64]>,
  meter: AudioMixerMeter,
  acoustics: AudioAcoustics,
  /// The listener's reverb on the mixer.
  reverb: Arc<ReverbParameters>,
  /// Voices let go of whose stop the audio thread had no room for, asked again by every call until it takes them.
  letting_go: Vec<AudioVoiceId>,
}

impl AudioEngine {
  pub(crate) fn new(
    commands: Producer<MixerCommand>,
    retired: Consumer<Box<Voice>>,
    retired_taps: Consumer<Box<MixerTap>>,
    shared: EngineShared,
  ) -> Self {
    let EngineShared {
      parameters,
      statuses,
      meter,
      acoustics,
      reverb,
    } = shared;
    let slots: VoiceSlots = VoiceSlots::new(statuses.len());
    let start_frames: Box<[u64]> = vec![0; statuses.len()].into_boxed_slice();

    Self {
      commands,
      retired,
      retired_taps,
      parameters,
      statuses,
      slots,
      start_frames,
      meter,
      acoustics,
      reverb,
      letting_go: Vec::new(),
    }
  }

  /// The mixer's clock and load.
  pub fn get_meter(&self) -> &AudioMixerMeter {
    &self.meter
  }

  /// The acoustics placed voices are heard through.
  pub fn get_acoustics(&self) -> &AudioAcoustics {
    &self.acoustics
  }

  /// Starts a voice, which the audio thread takes up with the next buffer.
  ///
  /// # Errors
  ///
  /// Refuses the voice when every slot plays or the audio thread has not taken its commands; the source is dropped.
  pub fn play(&mut self, play: AudioPlay) -> Result<AudioVoiceId, AudioRefusal> {
    self.start(play, None)
  }

  /// Starts a voice the acoustics place through `spatial`, heard as `parameters` say until they are set again.
  ///
  /// # Errors
  ///
  /// Refuses the voice as [`Self::play`] does.
  pub(crate) fn play_placed(
    &mut self,
    play: AudioPlay,
    spatial: Box<VoiceSpatial>,
    parameters: &VoiceSpatialParameters,
  ) -> Result<AudioVoiceId, AudioRefusal> {
    self.start(play, Some((spatial, parameters)))
  }

  /// Moves a placed voice to how `parameters` say it is heard, from its next block. A voice that has ended is left
  /// alone.
  pub(crate) fn set_voice_spatial(&mut self, voice: AudioVoiceId, parameters: &VoiceSpatialParameters) {
    if let Some(block) = self.get_parameters(voice) {
      block.set_spatial(parameters);
    }
  }

  /// Sets how the listener's reverb decays, each band's time in seconds; nought silences it.
  pub(crate) fn set_reverb_times(&self, times: [f32; 3]) {
    self.reverb.set_times(times);
  }

  fn start(
    &mut self,
    play: AudioPlay,
    spatial: Option<(Box<VoiceSpatial>, &VoiceSpatialParameters)>,
  ) -> Result<AudioVoiceId, AudioRefusal> {
    self.collect();

    let id: AudioVoiceId = self.slots.take().ok_or(AudioRefusal::VoicesExhausted)?;
    let parameters: &VoiceParameters = &self.parameters[id.slot as usize];

    // The slot is free, so no voice reads its block until this one starts with what is written here.
    parameters.set_gain(play.gain);
    parameters.set_rate(play.rate);
    parameters.set_pan(play.pan);
    self.start_frames[id.slot as usize] = play.source.get_start_frame();

    let spatial: Option<Box<VoiceSpatial>> = spatial.map(|(spatial, spatial_parameters)| {
      parameters.set_spatial(spatial_parameters);

      spatial
    });
    let voice: Box<Voice> = Box::new(Voice::new(id, play.source, play.bus, spatial).starting_at(play.at));

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

  /// Stops the voice for one that holds it no longer: a stop the audio thread has no room for is asked again by every
  /// later call, so a looped voice let go of never plays on unheld.
  pub fn let_go(&mut self, voice: AudioVoiceId) {
    if self.stop(voice).is_err() {
      self.letting_go.push(voice);
    }
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

  /// Moves the voice's left and right gains to `pan` from the next buffer. A voice that has ended is left alone.
  pub fn set_voice_pan(&mut self, voice: AudioVoiceId, pan: [f32; 2]) {
    if let Some(parameters) = self.get_parameters(voice) {
      parameters.set_pan(pan);
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

  /// Where the voice is. A voice whose slot has been freed has ended, whatever its last report said: asked first, since
  /// only this side gives a slot to its next voice, so while the voice holds it no other voice's report can be read.
  pub fn get_playback(&self, voice: AudioVoiceId) -> AudioVoicePlayback {
    if !self.slots.is_current(voice) {
      return AudioVoicePlayback {
        state: AudioVoiceState::Ended,
        frame: 0,
      };
    }

    self
      .statuses
      .get(voice.slot as usize)
      .and_then(|status| status.read(voice.generation))
      .unwrap_or_else(|| AudioVoicePlayback {
        state: AudioVoiceState::Starting,
        frame: self.start_frames[voice.slot as usize],
      })
  }

  /// Voices playing or waiting to start, of [`Self::get_voices_count`].
  pub fn get_playing_count(&self) -> usize {
    self.slots.get_playing_count()
  }

  pub fn get_voices_count(&self) -> usize {
    self.slots.get_count()
  }

  /// Taps the mix after the reverb and before the master from the mixer's next block, through a ring holding `length`
  /// of it, in place of any tap set before.
  ///
  /// # Errors
  ///
  /// Refuses when the audio thread has not taken its commands.
  pub fn open_tap(&mut self, length: Duration) -> Result<AudioTap, AudioRefusal> {
    let (tap, mixer_tap) = AudioTap::new(length, self.meter.get_rate());

    self.send(MixerCommand::SetTap(Box::new(mixer_tap)))?;

    Ok(tap)
  }

  /// Frees the voices the mixer is done with, and their slots, and the taps it let go of; asks again to stop the voices
  /// let go of that are still playing.
  pub fn collect(&mut self) {
    while let Ok(voice) = self.retired.pop() {
      self.slots.give_back(voice.id);
    }

    while self.retired_taps.pop().is_ok() {}

    let Self {
      letting_go,
      slots,
      commands,
      ..
    } = self;

    letting_go.retain(|voice| slots.is_current(*voice) && commands.push(MixerCommand::Stop(*voice)).is_err());
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
