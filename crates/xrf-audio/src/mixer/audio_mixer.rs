use std::sync::Arc;
use std::time::Instant;

use rtrb::{Consumer, Producer};

use crate::mixer::audio_bus::AudioBus;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;
use crate::mixer::gain_ramp::GainRamp;
use crate::mixer::mixer_command::MixerCommand;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::audio_voice_playback::AudioVoicePlayback;
use crate::voice::audio_voice_state::AudioVoiceState;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::shared::voice_status::VoiceStatus;
use crate::voice::voice::Voice;

/// The audio thread's half: a fixed pool of voice slots mixed into their buses, the buses into the master, and the
/// master into the device's buffer.
///
/// It takes no lock and allocates nothing while rendering: commands arrive through a ring, each slot's continuous
/// controls through its parameter block, voices arrive boxed, and a voice that ends goes back through another ring to
/// be freed by the control side.
pub struct AudioMixer {
  voices: Box<[Option<Box<Voice>>]>,
  commands: Consumer<MixerCommand>,
  retired: Producer<Box<Voice>>,
  /// Each slot's controls, written by the control side.
  parameters: Arc<[VoiceParameters]>,
  /// Each slot's report, read by the control side.
  statuses: Arc<[VoiceStatus]>,
  buses: [GainRamp; AudioBus::COUNT],
  master: GainRamp,
  /// Each bus's stereo pairs for one block.
  bus_buffers: [Box<[f32]>; AudioBus::COUNT],
  /// Where a voice renders its source for one block before writing it to its bus.
  voice_block: Box<[f32]>,
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
    parameters: Arc<[VoiceParameters]>,
    statuses: Arc<[VoiceStatus]>,
    meter: AudioMixerMeter,
  ) -> Self {
    let mut mixer: Self = Self {
      voices: (0..statuses.len()).map(|_| None).collect(),
      commands,
      retired,
      parameters,
      statuses,
      buses: [GainRamp::new(1.0); AudioBus::COUNT],
      master: GainRamp::new(1.0),
      bus_buffers: std::array::from_fn(|_| vec![0.0; Self::BLOCK_FRAMES * 2].into_boxed_slice()),
      voice_block: vec![0.0; Self::BLOCK_FRAMES * 2].into_boxed_slice(),
      output_rate: Self::DEFAULT_OUTPUT_RATE,
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

  /// Renders at `rate` from the next buffer on; the device sets it to the rate it opened with.
  pub fn set_output_rate(&mut self, rate: u32) {
    self.output_rate = rate.max(1);
    self.meter.set_rate(self.output_rate);
    self.ramp_frames = Self::to_frames(Self::RAMP_SECONDS, self.output_rate);
    self.fade_frames = Self::to_frames(Self::FADE_SECONDS, self.output_rate);
  }

  /// Fills `out`, interleaved frames of `channels`, with the next frames of every voice: the first two channels carry
  /// the mix, any further ones silence, and a single channel the two averaged.
  ///
  /// Commands and parameters are taken before each block, so a command waits at most [`Self::BLOCK_FRAMES`] however
  /// long the device's buffer is.
  pub fn render(&mut self, out: &mut [f32], channels: usize) {
    let started: Instant = Instant::now();
    let channels: usize = channels.max(1);
    let frames: usize = out.len() / channels;

    for chunk in out.chunks_mut(Self::BLOCK_FRAMES * channels) {
      self.take_commands();
      self.follow_parameters();
      self.render_block(chunk, channels);
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

  fn render_block(&mut self, out: &mut [f32], channels: usize) {
    let frames: usize = out.len() / channels;

    for buffer in &mut self.bus_buffers {
      buffer[..frames * 2].fill(0.0);
    }

    for slot in &mut self.voices {
      let Some(voice) = slot else {
        continue;
      };

      if !voice.mix(
        &mut self.bus_buffers[voice.bus.get_index()][..frames * 2],
        &mut self.voice_block,
        self.output_rate,
      ) {
        Self::retire(slot, &mut self.retired, &self.statuses);
      }
    }

    for (frame, samples) in out.chunks_exact_mut(channels).enumerate() {
      let mut left: f32 = 0.0;
      let mut right: f32 = 0.0;

      for (bus, buffer) in self.buses.iter_mut().zip(&self.bus_buffers) {
        let gain: f32 = bus.next();

        left += buffer[frame * 2] * gain;
        right += buffer[frame * 2 + 1] * gain;
      }

      let master: f32 = self.master.next();

      if channels == 1 {
        samples[0] = (left + right) * 0.5 * master;
      } else {
        samples[0] = left * master;
        samples[1] = right * master;
        samples[2..].fill(0.0);
      }
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
      }
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

  fn to_frames(seconds: f64, rate: u32) -> u32 {
    (seconds * f64::from(rate)).round() as u32
  }
}
