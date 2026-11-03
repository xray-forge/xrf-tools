use crate::mixer::audio_bus::AudioBus;
use crate::mixer::gain_ramp::GainRamp;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::audio_voice_state::AudioVoiceState;
use crate::voice::cursor::voice_cursor::VoiceCursor;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::voice_source::VoiceSource;

/// A sound playing on the audio thread. Built on the control thread and boxed there, so the audio thread only moves it
/// in and out of its slot and frees nothing.
///
/// Each block it renders its source, resampled, into a block of the source's own channels, then writes that block to
/// its bus: what the sound is and where it is heard stay apart, so what shapes the second leaves the first alone. A
/// voice whose gain has settled at silence only moves on through its source.
#[derive(Debug)]
pub(crate) struct Voice {
  pub(crate) id: AudioVoiceId,
  pub(crate) bus: AudioBus,
  cursor: VoiceCursor,
  /// Follows the slot's [`VoiceParameters`].
  gain: GainRamp,
  /// The left and right outputs' gains, following the slot's [`VoiceParameters`].
  pan: [GainRamp; 2],
  /// Brings a resumed voice in and a paused or stopped one out without a click.
  fade: GainRamp,
  /// Playback speed, above zero, 1 at the source's own rate, following the slot's [`VoiceParameters`].
  rate: f32,
  motion: VoiceMotion,
}

/// Whether a voice moves through its source, and where a fade is taking it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VoiceMotion {
  Playing,
  /// Fading out, to hold its place once silent.
  Pausing,
  Paused,
  /// Fading out, to end once silent.
  Stopping,
}

impl Voice {
  /// A voice at its slot's parameters, which the mixer applies as it takes the voice up.
  pub(crate) fn new(id: AudioVoiceId, source: VoiceSource, bus: AudioBus) -> Self {
    Self {
      id,
      bus,
      cursor: VoiceCursor::from(source),
      gain: GainRamp::new(1.0),
      pan: [GainRamp::new(1.0); 2],
      fade: GainRamp::new(1.0),
      rate: 1.0,
      motion: VoiceMotion::Playing,
    }
  }

  pub(crate) fn get_state(&self) -> AudioVoiceState {
    match self.motion {
      VoiceMotion::Playing | VoiceMotion::Pausing | VoiceMotion::Stopping => AudioVoiceState::Playing,
      VoiceMotion::Paused => AudioVoiceState::Paused,
    }
  }

  pub(crate) fn get_frame(&self) -> u64 {
    self.cursor.get_frame()
  }

  /// Moves towards the slot's parameters, a new gain over `frames` frames.
  pub(crate) fn follow(&mut self, parameters: &VoiceParameters, frames: u32) {
    let gain: f32 = parameters.get_gain();

    if gain != self.gain.get_target() {
      self.gain.set_target(gain, frames);
    }

    for (ramp, gain) in self.pan.iter_mut().zip(parameters.get_pan()) {
      if gain != ramp.get_target() {
        ramp.set_target(gain, frames);
      }
    }

    self.rate = parameters.get_rate();
  }

  pub(crate) fn pause(&mut self, frames: u32) {
    if self.motion == VoiceMotion::Playing {
      self.motion = VoiceMotion::Pausing;
      self.fade.set_target(0.0, frames);
    }
  }

  pub(crate) fn resume(&mut self, frames: u32) {
    if matches!(self.motion, VoiceMotion::Pausing | VoiceMotion::Paused) {
      self.motion = VoiceMotion::Playing;
      self.fade.set_target(1.0, frames);
    }
  }

  pub(crate) fn stop(&mut self, frames: u32) {
    self.motion = VoiceMotion::Stopping;
    self.fade.set_target(0.0, frames);
  }

  /// Adds the voice's next frames to the stereo pairs of `out` at `output_rate`, rendering its source into `block`,
  /// scratch of at least two samples a frame. Answers `false` once it has played out or faded out after a stop, when
  /// the mixer retires it.
  pub(crate) fn mix(&mut self, out: &mut [f32], block: &mut [f32], output_rate: u32) -> bool {
    if self.motion == VoiceMotion::Paused {
      return true;
    }

    let frames: usize = out.len() / 2;
    let step: f64 = f64::from(self.cursor.get_sample_rate()) * f64::from(self.rate) / f64::from(output_rate);
    let is_playing: bool = if self.is_silent() {
      self.fade.skip(frames as u32);
      self.cursor.skip(frames, step)
    } else {
      let channels: usize = self.cursor.get_channels().get_count();
      let block: &mut [f32] = &mut block[..frames * channels];

      block.fill(0.0);

      let is_playing: bool = self.cursor.render(block, step);

      self.write_output(block, channels, out);

      is_playing
    };

    if self.fade.is_settled() && self.fade.get_target() == 0.0 {
      match self.motion {
        VoiceMotion::Pausing => self.motion = VoiceMotion::Paused,
        VoiceMotion::Stopping => return false,
        VoiceMotion::Playing | VoiceMotion::Paused => {}
      }
    }

    is_playing
  }

  /// Whether nothing of it is heard until its gain changes: settled at silence, or on both sides.
  fn is_silent(&self) -> bool {
    self.gain.is_silent() || self.pan.iter().all(GainRamp::is_silent)
  }

  /// Adds `block`, frames of `channels`, to the stereo pairs of `out` at the voice's gain, pan and fade: a mono source
  /// is heard in both through the pan's two gains.
  fn write_output(&mut self, block: &[f32], channels: usize, out: &mut [f32]) {
    let [left, right] = &mut self.pan;

    // Settled, as most voices are most of the time, every frame takes the same two gains.
    if self.gain.is_settled() && self.fade.is_settled() && left.is_settled() && right.is_settled() {
      let scale: f32 = self.gain.get_value() * self.fade.get_value();
      let (left_gain, right_gain): (f32, f32) = (scale * left.get_value(), scale * right.get_value());
      let pairs: &mut [[f32; 2]] = out.as_chunks_mut::<2>().0;

      if channels == 1 {
        for (pair, sample) in pairs.iter_mut().zip(block) {
          pair[0] += sample * left_gain;
          pair[1] += sample * right_gain;
        }
      } else {
        for (pair, frame) in pairs.iter_mut().zip(block.as_chunks::<2>().0) {
          pair[0] += frame[0] * left_gain;
          pair[1] += frame[1] * right_gain;
        }
      }

      return;
    }

    for (frame, pair) in out.as_chunks_mut::<2>().0.iter_mut().enumerate() {
      let scale: f32 = self.gain.next() * self.fade.next();
      let (left_gain, right_gain): (f32, f32) = (scale * left.next(), scale * right.next());

      if channels == 1 {
        let sample: f32 = block[frame];

        pair[0] += sample * left_gain;
        pair[1] += sample * right_gain;
      } else {
        pair[0] += block[frame * 2] * left_gain;
        pair[1] += block[frame * 2 + 1] * right_gain;
      }
    }
  }
}
