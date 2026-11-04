use audionimbus::{
  AudioBufferMut, AudioBufferRef, AudioSettings, BinauralEffect, BinauralEffectParams, BinauralEffectSettings, Context,
  DirectEffect, DirectEffectParams, DirectEffectSettings, Direction, Equalizer, Hrtf, HrtfInterpolation, PanningEffect,
  PanningEffectParams, PanningEffectSettings, SpeakerLayout, Transmission,
};

use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::acoustics::effects::ear_blocks::EarBlocks;
use crate::acoustics::effects::placed_block::PlacedBlock;
use crate::mixer::audio_mixer::AudioMixer;
use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;

/// A placed voice's Steam Audio effects, made on the control side and run on the audio thread a block at a time: the
/// direct path (distance, air, occlusion and what the walls let through), then both ears, through the HRTF or panned.
pub(crate) struct VoiceEffects {
  direct: DirectEffect,
  binaural: BinauralEffect,
  /// Kept whole, so a block only moves its direction rather than taking another hold on the HRTF.
  binaural_params: BinauralEffectParams,
  panning: PanningEffect,
  /// The block after the direct path.
  direct_out: Box<[f32]>,
  /// Both ears, the left's frames then the right's.
  ears: Box<[f32]>,
  /// Both ears through the way the last block took, faded out over a block that switches between the HRTF and panning.
  fading: Box<[f32]>,
  /// How the last block reached the ears, none before the first.
  last_ears: Option<AudioSpatialization>,
}

impl std::fmt::Debug for VoiceEffects {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.debug_struct("VoiceEffects").finish_non_exhaustive()
  }
}

impl VoiceEffects {
  pub(crate) fn new(context: &Context, settings: &AudioSettings, hrtf: Hrtf) -> Result<Self, String> {
    let failed = |what: &str, error: audionimbus::SteamAudioError| format!("Steam Audio's {what} is not made: {error}");

    Ok(Self {
      direct: DirectEffect::try_new(context, settings, &DirectEffectSettings { num_channels: 1 })
        .map_err(|error| failed("direct effect", error))?,
      binaural: BinauralEffect::try_new(context, settings, &BinauralEffectSettings { hrtf: hrtf.clone() })
        .map_err(|error| failed("binaural effect", error))?,
      binaural_params: BinauralEffectParams {
        direction: Direction::new(0.0, 0.0, -1.0),
        interpolation: HrtfInterpolation::Bilinear,
        spatial_blend: 1.0,
        hrtf,
        peak_delays: None,
      },
      panning: PanningEffect::try_new(
        context,
        settings,
        &PanningEffectSettings {
          speaker_layout: SpeakerLayout::Stereo,
        },
      )
      .map_err(|error| failed("panning effect", error))?,
      direct_out: vec![0.0; AudioMixer::BLOCK_FRAMES].into_boxed_slice(),
      ears: vec![0.0; AudioMixer::BLOCK_FRAMES * 2].into_boxed_slice(),
      fading: vec![0.0; AudioMixer::BLOCK_FRAMES * 2].into_boxed_slice(),
      last_ears: None,
    })
  }

  /// Renders one mono block of [`AudioMixer::BLOCK_FRAMES`] frames through the direct path and onto both ears. A block
  /// that switches between the HRTF and panning crosses from the old way to the new over its frames, the new way
  /// started afresh rather than from what it last heard, so a voice taking or losing one of the HRTF's places neither
  /// clicks nor replays an old tail.
  pub(crate) fn apply(&mut self, parameters: &VoiceSpatialParameters, mono: &[f32]) -> PlacedBlock<'_> {
    let direct: DirectEffectParams = DirectEffectParams {
      distance_attenuation: Some(parameters.attenuation),
      air_absorption: Some(Equalizer(parameters.air_absorption)),
      directivity: None,
      occlusion: Some(parameters.occlusion),
      transmission: Some(Transmission::FrequencyDependent(Equalizer(parameters.transmission))),
    };
    let [x, y, z] = parameters.direction;

    // Buffers of the block's length and the effects' channel counts are always accepted, so the answers carry nothing.
    if let (Ok(input), Ok(mut output)) = (
      AudioBufferRef::try_from(mono),
      AudioBufferMut::try_from(&mut self.direct_out[..]),
    ) {
      let _ = self.direct.apply(&direct, &input, &mut output);
    }

    let ears: AudioSpatialization = if parameters.is_binaural {
      AudioSpatialization::Headphones
    } else {
      AudioSpatialization::Speakers
    };
    let switched: Option<AudioSpatialization> = self.last_ears.replace(ears).filter(|last| *last != ears);

    if let Some(last) = switched {
      self.render_ears(last, [x, y, z]);
      self.fading.copy_from_slice(&self.ears);

      match ears {
        AudioSpatialization::Headphones => self.binaural.reset(),
        AudioSpatialization::Speakers => self.panning.reset(),
      }
    }

    self.render_ears(ears, [x, y, z]);

    if switched.is_some() {
      self.cross_from_fading();
    }

    let (left, right) = self.ears.split_at(AudioMixer::BLOCK_FRAMES);

    PlacedBlock {
      direct: &self.direct_out,
      ears: EarBlocks { left, right },
    }
  }

  /// Puts the direct path's block onto both ears as `ears` says, from `direction` in the listener's space.
  fn render_ears(&mut self, ears: AudioSpatialization, [x, y, z]: [f32; 3]) {
    // Buffers of the block's length and the effects' channel counts are always accepted, so the answers carry nothing.
    if let (Ok(input), Ok(mut output)) = (
      AudioBufferRef::try_from(&self.direct_out[..]),
      AudioBufferMut::try_new(&mut self.ears[..], 2),
    ) {
      match ears {
        AudioSpatialization::Headphones => {
          self.binaural_params.direction = Direction::new(x, y, z);
          let _ = self.binaural.apply(&self.binaural_params, &input, &mut output);
        }
        AudioSpatialization::Speakers => {
          let panning: PanningEffectParams = PanningEffectParams {
            direction: Direction::new(x, y, z),
          };

          let _ = self.panning.apply(&panning, &input, &mut output);
        }
      }
    }
  }

  /// Crosses each ear from the faded way's block to the new one's over the block's frames.
  fn cross_from_fading(&mut self) {
    let frames: usize = AudioMixer::BLOCK_FRAMES;

    for (ear, faded) in self.ears.chunks_exact_mut(frames).zip(self.fading.chunks_exact(frames)) {
      for (frame, (sample, old)) in ear.iter_mut().zip(faded).enumerate() {
        let share: f32 = (frame + 1) as f32 / frames as f32;

        *sample = old + (*sample - old) * share;
      }
    }
  }
}
