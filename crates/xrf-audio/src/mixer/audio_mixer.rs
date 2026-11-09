use std::sync::Arc;
use std::time::Instant;

use rtrb::{Consumer, Producer};

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::effects::ear_blocks::EarBlocks;
use crate::control::engine::engine_shared::EngineShared;
use crate::mixer::gain_ramp::GainRamp;
use crate::mixer::meter::audio_mixer_meter::AudioMixerMeter;
use crate::mixer::mixer_command::MixerCommand;
use crate::mixer::reverb::mixer_reverb::MixerReverb;
use crate::mixer::reverb::reverb_parameters::ReverbParameters;
use crate::mixer::tap::mixer_tap::MixerTap;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::playback::audio_voice_playback::AudioVoicePlayback;
use crate::voice::playback::audio_voice_state::AudioVoiceState;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::shared::voice_status::VoiceStatus;
use crate::voice::voice::Voice;
use xrf_audio_scene::AudioBus;

/// The audio thread's half: a fixed pool of voice slots mixed into their buses, the buses and the listener's reverb into
/// the master, and the master into the device's buffer; a tap takes the mix before the master.
pub struct AudioMixer {
  voices: Box<[Option<Box<Voice>>]>,
  commands: Consumer<MixerCommand>,
  retired: Producer<Box<Voice>>,
  /// The tap the mix goes into before the master, and where one let go or replaced goes back to be freed.
  tap: Option<Box<MixerTap>>,
  retired_taps: Producer<Box<MixerTap>>,
  /// Each slot's controls, written by the control side.
  parameters: Arc<[VoiceParameters]>,
  /// Each slot's report, read by the control side.
  statuses: Arc<[VoiceStatus]>,
  buses: [GainRamp; AudioBus::COUNT],
  master: GainRamp,
  /// Each bus's stereo pairs for one block.
  bus_buffers: [Box<[f32]>; AudioBus::COUNT],
  /// What each bus's voices send the listener's reverb in one block, taken at the bus's gain.
  reverb_sends: [Box<[f32]>; AudioBus::COUNT],
  /// What the reverb hears in one block.
  reverb_input: Box<[f32]>,
  reverb: Option<Box<MixerReverb>>,
  /// How the reverb decays, written by the control side.
  reverb_parameters: Arc<ReverbParameters>,
  acoustics: AudioAcoustics,
  /// Where a voice renders its source for one block before writing it to its bus.
  voice_block: Box<[f32]>,
  /// The last block rendered, in stereo pairs, which buffers are filled from.
  block: Box<[f32]>,
  /// Frames of [`Self::block`] already handed out.
  block_read: usize,
  /// The mixer frame the next block starts at: every block rendered so far, the clock voices start on.
  block_frame: u64,
  output_rate: u32,
  /// Frames a gain change takes.
  ramp_frames: u32,
  /// Frames a pause, resume or stop fades over.
  fade_frames: u32,
  meter: AudioMixerMeter,
}

impl AudioMixer {
  /// Frames mixed at a time: a device's longer buffer is rendered in blocks of this many.
  pub const BLOCK_FRAMES: usize = 256;

  /// The rate the mixer renders at before a device or an offline render sets its own.
  pub const DEFAULT_OUTPUT_RATE: u32 = 48_000;

  /// Seconds a volume change takes, short enough to follow a slider and long enough not to click.
  const RAMP_SECONDS: f64 = 0.010;

  /// Seconds a pause, resume or stop fades over.
  const FADE_SECONDS: f64 = 0.005;

  pub(crate) fn new(
    commands: Consumer<MixerCommand>,
    retired: Producer<Box<Voice>>,
    retired_taps: Producer<Box<MixerTap>>,
    shared: EngineShared,
  ) -> Self {
    let EngineShared {
      parameters,
      statuses,
      meter,
      acoustics,
      reverb: reverb_parameters,
    } = shared;
    let block = || vec![0.0; Self::BLOCK_FRAMES * 2].into_boxed_slice();
    let mut mixer: Self = Self {
      voices: (0..statuses.len()).map(|_| None).collect(),
      commands,
      retired,
      tap: None,
      retired_taps,
      parameters,
      statuses,
      buses: [GainRamp::new(1.0); AudioBus::COUNT],
      master: GainRamp::new(1.0),
      bus_buffers: std::array::from_fn(|_| block()),
      reverb_sends: std::array::from_fn(|_| vec![0.0; Self::BLOCK_FRAMES].into_boxed_slice()),
      reverb_input: vec![0.0; Self::BLOCK_FRAMES].into_boxed_slice(),
      reverb: None,
      reverb_parameters,
      acoustics,
      voice_block: block(),
      block: block(),
      block_read: Self::BLOCK_FRAMES,
      block_frame: 0,
      output_rate: 0,
      ramp_frames: 0,
      fade_frames: 0,
      meter,
    };

    mixer.set_output_rate(Self::DEFAULT_OUTPUT_RATE);
    mixer
  }

  pub fn get_output_rate(&self) -> u32 {
    self.output_rate
  }

  /// Renders at `rate` from the next buffer on; the device sets it to the rate it opened with, before it plays, which
  /// is when the reverb is made again for a new rate, and a tap's stream ends.
  pub fn set_output_rate(&mut self, rate: u32) {
    let rate: u32 = rate.max(1);

    if rate != self.output_rate || (self.reverb.is_none() && self.acoustics.is_running()) {
      self.reverb = MixerReverb::for_acoustics(&self.acoustics, rate);
    }

    if let Some(tap) = &self.tap {
      tap.follow_rate(rate);
    }

    self.output_rate = rate;
    self.meter.set_rate(self.output_rate);
    self.ramp_frames = Self::get_frames(Self::RAMP_SECONDS, self.output_rate);
    self.fade_frames = Self::get_frames(Self::FADE_SECONDS, self.output_rate);
  }

  /// Fills `out`, interleaved frames of `channels`, with the next frames of every voice: the first two channels carry
  /// the mix, any further ones silence, and a single channel the two averaged.
  ///
  /// Commands and parameters are taken before each block, so a command waits at most [`Self::BLOCK_FRAMES`] however
  /// long the device's buffer is; what a buffer leaves of a block plays at the start of the next.
  pub fn render(&mut self, out: &mut [f32], channels: usize) {
    let started: Instant = Instant::now();
    let channels: usize = channels.max(1);
    let frames: usize = out.len() / channels;
    let mut written: usize = 0;

    while written < frames {
      if self.block_read == Self::BLOCK_FRAMES {
        self.take_commands();
        self.follow_parameters();
        self.render_block();
        self.block_read = 0;
        self.block_frame += Self::BLOCK_FRAMES as u64;
      }

      let count: usize = (Self::BLOCK_FRAMES - self.block_read).min(frames - written);
      let pairs: &[[f32; 2]] = &self.block.as_chunks::<2>().0[self.block_read..self.block_read + count];

      for (samples, [left, right]) in out[written * channels..(written + count) * channels]
        .chunks_exact_mut(channels)
        .zip(pairs)
      {
        if channels == 1 {
          samples[0] = (left + right) * 0.5;
        } else {
          samples[0] = *left;
          samples[1] = *right;
          samples[2..].fill(0.0);
        }
      }

      self.block_read += count;
      written += count;
    }

    let voices: u32 = self.report();
    let playing_for: f64 = frames as f64 / f64::from(self.output_rate);
    let load: f64 = if frames == 0 {
      0.0
    } else {
      started.elapsed().as_secs_f64() / playing_for
    };

    self.meter.record(frames as u64, load as f32, voices);
  }

  /// The mixer's clock and load, for whoever drives or watches it.
  pub fn get_meter(&self) -> &AudioMixerMeter {
    &self.meter
  }

  /// Renders the next block into [`Self::block`]: every voice into its bus, the buses and what they send the reverb at
  /// their gains, the reverb, then the master.
  fn render_block(&mut self) {
    for buffer in &mut self.bus_buffers {
      buffer.fill(0.0);
    }

    for send in &mut self.reverb_sends {
      send.fill(0.0);
    }

    for slot in &mut self.voices {
      let Some(voice) = slot else {
        continue;
      };
      let bus: usize = voice.bus.get_index();

      if !voice.mix(
        &mut self.bus_buffers[bus],
        &mut self.reverb_sends[bus],
        &mut self.voice_block,
        self.output_rate,
      ) {
        Self::retire(slot, &mut self.retired, &self.statuses);
      }
    }

    self.reverb_input.fill(0.0);

    for (frame, pair) in self.block.as_chunks_mut::<2>().0.iter_mut().enumerate() {
      let mut left: f32 = 0.0;
      let mut right: f32 = 0.0;
      let mut sent: f32 = 0.0;

      for ((bus, buffer), send) in self.buses.iter_mut().zip(&self.bus_buffers).zip(&self.reverb_sends) {
        let gain: f32 = bus.next();

        left += buffer[frame * 2] * gain;
        right += buffer[frame * 2 + 1] * gain;
        sent += send[frame] * gain;
      }

      *pair = [left, right];
      self.reverb_input[frame] = sent;
    }

    if let Some(reverb) = self.reverb.as_deref_mut() {
      let EarBlocks { left, right } = reverb.apply(self.reverb_parameters.get_times(), &self.reverb_input);

      for ((pair, left), right) in self.block.as_chunks_mut::<2>().0.iter_mut().zip(left).zip(right) {
        pair[0] += left;
        pair[1] += right;
      }
    }

    if let Some(tap) = self.tap.as_deref_mut() {
      tap.push(self.block_frame, &self.block);
    }

    for pair in self.block.as_chunks_mut::<2>().0 {
      let master: f32 = self.master.next();

      pair[0] *= master;
      pair[1] *= master;
    }
  }

  fn take_commands(&mut self) {
    let fade_frames: u32 = self.fade_frames;
    let ramp_frames: u32 = self.ramp_frames;

    while let Ok(command) = self.commands.pop() {
      match command {
        MixerCommand::Start(mut voice) => {
          let slot: usize = voice.id.slot as usize;

          voice.follow(&self.parameters[slot], 0);
          voice.take_up(self.block_frame, self.output_rate);
          self.statuses[slot].write(voice.id.generation, Self::get_playback(&voice));
          self.voices[slot] = Some(voice);
        }
        MixerCommand::Stop(id) => {
          if let Some(voice) = self.find_voice(id) {
            voice.stop(fade_frames);
          }
        }
        MixerCommand::Pause(id) => {
          if let Some(voice) = self.find_voice(id) {
            voice.pause(fade_frames);
          }
        }
        MixerCommand::Resume(id) => {
          if let Some(voice) = self.find_voice(id) {
            voice.resume(fade_frames);
          }
        }
        MixerCommand::SetBusGain { bus, gain } => self.buses[bus.get_index()].set_target(gain, ramp_frames),
        MixerCommand::SetMasterGain(gain) => self.master.set_target(gain, ramp_frames),
        MixerCommand::SetTap(tap) => {
          tap.start(self.output_rate);
          Self::retire_tap(self.tap.replace(tap), &mut self.retired_taps);
        }
      }
    }

    if self.tap.as_ref().is_some_and(|tap| tap.is_abandoned()) {
      Self::retire_tap(self.tap.take(), &mut self.retired_taps);
    }
  }

  /// Hands `tap` back to the control side, which frees it.
  fn retire_tap(tap: Option<Box<MixerTap>>, retired: &mut Producer<Box<MixerTap>>) {
    // The control side never has more taps out than the ring holds (`AudioEngine::TAPS_MOST`), so it is never full; were
    // it, the tap would be lost rather than freed on this thread.
    if let Some(tap) = tap
      && let Err(rtrb::PushError::Full(tap)) = retired.push(tap)
    {
      std::mem::forget(tap);
    }
  }

  /// Moves every playing voice towards its slot's parameters, once a block.
  fn follow_parameters(&mut self) {
    for (voice, parameters) in self.voices.iter_mut().zip(self.parameters.iter()) {
      if let Some(voice) = voice {
        voice.follow(parameters, self.ramp_frames);
      }
    }
  }

  /// Publishes where every playing voice is, once a buffer, answering how many there are.
  fn report(&self) -> u32 {
    let mut count: u32 = 0;

    for voice in self.voices.iter().flatten() {
      self.statuses[voice.id.slot as usize].write(voice.id.generation, Self::get_playback(voice));
      count += 1;
    }

    count
  }

  fn find_voice(&mut self, id: AudioVoiceId) -> Option<&mut Box<Voice>> {
    self
      .voices
      .get_mut(id.slot as usize)?
      .as_mut()
      .filter(|voice| voice.id.generation == id.generation)
  }

  /// Hands the voice in `slot` back to the control side, which frees it and its slot.
  fn retire(slot: &mut Option<Box<Voice>>, retired: &mut Producer<Box<Voice>>, statuses: &[VoiceStatus]) {
    let Some(voice) = slot.take() else {
      return;
    };

    statuses[voice.id.slot as usize].write(
      voice.id.generation,
      AudioVoicePlayback {
        state: AudioVoiceState::Ended,
        frame: voice.get_frame(),
      },
    );

    // The ring holds one entry a slot and a slot is reused only after its voice came back through it, so it is never
    // full here; were it full, the voice would be lost rather than freed on this thread.
    if let Err(rtrb::PushError::Full(voice)) = retired.push(voice) {
      std::mem::forget(voice);
    }
  }

  fn get_playback(voice: &Voice) -> AudioVoicePlayback {
    AudioVoicePlayback {
      state: voice.get_state(),
      frame: voice.get_frame(),
    }
  }

  fn get_frames(seconds: f64, rate: u32) -> u32 {
    (seconds * f64::from(rate)).round() as u32
  }
}
