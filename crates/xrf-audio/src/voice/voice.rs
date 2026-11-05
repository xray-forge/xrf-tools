use crate::acoustics::effects::ear_blocks::EarBlocks;
use crate::acoustics::effects::placed_block::PlacedBlock;
use crate::mixer::gain_ramp::GainRamp;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::cursor::voice_cursor::VoiceCursor;
use crate::voice::playback::audio_voice_state::AudioVoiceState;
use crate::voice::shared::voice_parameters::VoiceParameters;
use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;
use crate::voice::voice_source::VoiceSource;
use crate::voice::voice_spatial::VoiceSpatial;
use xrf_audio_scene::AudioBus;

/// A sound playing on the audio thread. Built on the control thread and boxed there, so the audio thread only moves it
/// in and out of its slot and frees nothing.
///
/// Each block it renders its source, resampled, into a block of the source's own channels, then writes that block to
/// its bus: what the sound is and where it is heard stay apart, so what shapes the second leaves the first alone. A
/// voice the acoustics place takes the second step through their effects and sends the listener's reverb its share; a
/// dry one takes its two pan gains. A voice whose gain has settled at silence only moves on through its source.
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
  /// The acoustics' effects of a placed voice.
  spatial: Option<Box<VoiceSpatial>>,
  /// How the effects hear it, following the slot's [`VoiceParameters`].
  spatial_parameters: VoiceSpatialParameters,
  /// The mixer frame its first frame plays at, until the mixer takes it up; none plays it from the block it is taken
  /// up in.
  at: Option<u64>,
  /// Frames of silence left before its first frame, rendered as nothing.
  delay: u32,
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
  /// A voice at its slot's parameters, which the mixer applies as it takes the voice up, placed through `spatial`
  /// where the acoustics run.
  pub(crate) fn new(id: AudioVoiceId, source: VoiceSource, bus: AudioBus, spatial: Option<Box<VoiceSpatial>>) -> Self {
    Self {
      id,
      bus,
      cursor: VoiceCursor::from(source),
      gain: GainRamp::new(1.0),
      pan: [GainRamp::new(1.0); 2],
      fade: GainRamp::new(1.0),
      rate: 1.0,
      motion: VoiceMotion::Playing,
      spatial,
      spatial_parameters: VoiceSpatialParameters::default(),
      at: None,
      delay: 0,
    }
  }

  /// The voice with its first frame playing at mixer frame `at`, none for the block the mixer takes it up in.
  pub(crate) fn starting_at(self, at: Option<u64>) -> Self {
    Self { at, ..self }
  }

  /// Lines the voice up with the mixer's clock as the mixer takes it up for the block starting at mixer frame `block`
  /// at `output_rate`: a start still ahead waits that many frames as silence; one already past moves on through its
  /// source as far as it would have played, so it plays where its clock says rather than late.
  pub(crate) fn take_up(&mut self, block: u64, output_rate: u32) {
    let Some(at) = self.at.take() else {
      return;
    };

    match at.checked_sub(block) {
      Some(ahead) => self.delay = u32::try_from(ahead).unwrap_or(u32::MAX),
      None => {
        let late: usize = usize::try_from(block - at).unwrap_or(usize::MAX);

        self.cursor.skip(late, self.get_step(output_rate));
      }
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

    if self.spatial.is_some() {
      self.spatial_parameters = parameters.get_spatial();
    }
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

  /// Adds the voice's next frames to the stereo pairs of `out` at `output_rate`, and what it sends the listener's
  /// reverb to `reverb`, one sample a frame, rendering its source into `block`, scratch of at least two samples a frame.
  /// A placed voice takes whole mixer blocks. Frames before a start still ahead are silence. Answers `false` once it
  /// has played out or faded out after a stop, when the mixer retires it.
  pub(crate) fn mix(&mut self, out: &mut [f32], reverb: &mut [f32], block: &mut [f32], output_rate: u32) -> bool {
    if self.motion == VoiceMotion::Paused {
      return true;
    }

    // Stopped before it was heard, it has nothing to fade.
    if self.delay > 0 && self.motion == VoiceMotion::Stopping {
      return false;
    }

    let frames: usize = out.len() / 2;
    let waited: usize = (self.delay as usize).min(frames);

    self.delay -= waited as u32;

    if waited == frames {
      return true;
    }

    let step: f64 = self.get_step(output_rate);
    let is_playing: bool = if self.is_silent() {
      self.fade.skip((frames - waited) as u32);
      self.cursor.skip(frames - waited, step)
    } else {
      let channels: usize = self.cursor.get_channels().get_count();
      let block: &mut [f32] = &mut block[..frames * channels];

      block.fill(0.0);

      let is_playing: bool = self.cursor.render(&mut block[waited * channels..], step);

      if self.spatial.is_some() {
        self.write_placed(block, channels, out, reverb);
      } else {
        self.write_output(block, channels, out);
      }

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

  /// Source frames an output frame at `output_rate` moves on by.
  fn get_step(&self, output_rate: u32) -> f64 {
    f64::from(self.cursor.get_sample_rate()) * f64::from(self.rate) / f64::from(output_rate)
  }

  /// Whether nothing of it is heard until its gain changes: settled at silence, or a dry one on both sides.
  fn is_silent(&self) -> bool {
    self.gain.is_silent() || (self.spatial.is_none() && self.pan.iter().all(GainRamp::is_silent))
  }

  /// Adds `block`, frames of `channels`, to the stereo pairs of `out` through the acoustics' effects, at the voice's
  /// gain and fade, and its share to `reverb`: a placed sound is mono, a stereo one heard as both channels averaged.
  fn write_placed(&mut self, block: &mut [f32], channels: usize, out: &mut [f32], reverb: &mut [f32]) {
    let Some(spatial) = self.spatial.as_deref_mut() else {
      return;
    };
    let frames: usize = out.len() / 2;

    if channels == 2 {
      // Each frame lands at or before where it was read, so averaging in place reads every pair before it is written.
      for frame in 0..frames {
        block[frame] = (block[frame * 2] + block[frame * 2 + 1]) * 0.5;
      }
    }

    let send: f32 = self.spatial_parameters.reverb_send;
    let PlacedBlock {
      direct,
      ears: EarBlocks { left, right },
    } = spatial.apply(&self.spatial_parameters, &block[..frames]);
    let pairs: &mut [[f32; 2]] = out.as_chunks_mut::<2>().0;

    for (((pair, left), right), (direct, reverb)) in pairs
      .iter_mut()
      .zip(left)
      .zip(right)
      .zip(direct.iter().zip(reverb.iter_mut()))
    {
      let scale: f32 = self.gain.next() * self.fade.next();

      pair[0] += left * scale;
      pair[1] += right * scale;
      *reverb += direct * scale * send;
    }
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
