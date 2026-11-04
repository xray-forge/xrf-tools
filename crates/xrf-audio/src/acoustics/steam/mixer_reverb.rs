use audionimbus::{
  AudioBufferMut, AudioBufferRef, AudioSettings, Context, Parametric, ReflectionEffect, ReflectionEffectParams,
  ReflectionEffectSettings,
};

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::effects::ear_blocks::EarBlocks;
use crate::mixer::audio_mixer::AudioMixer;

/// The listener's reverb on the audio thread: what every voice sends it, a mono block, rendered by Steam Audio's
/// parametric reverb at the decay times the scene's simulation last heard around the listener.
pub(crate) struct MixerReverb {
  effect: ReflectionEffect<Parametric>,
  /// The reverb, the left's frames then the right's.
  out: Box<[f32]>,
  impulse_response_size: u32,
}

impl MixerReverb {
  /// The longest reverb it renders, in seconds.
  pub(crate) const MAX_DURATION: f32 = 4.0;

  pub(crate) fn new(context: &Context, settings: &AudioSettings) -> Result<Self, String> {
    let impulse_response_size: u32 = (Self::MAX_DURATION * settings.sampling_rate as f32) as u32;
    let effect: ReflectionEffect<Parametric> = ReflectionEffect::try_new(
      context,
      settings,
      &ReflectionEffectSettings {
        impulse_response_size,
        num_channels: 2,
      },
    )
    .map_err(|error| format!("Steam Audio's reverb is not made: {error}"))?;

    Ok(Self {
      effect,
      out: vec![0.0; AudioMixer::BLOCK_FRAMES * 2].into_boxed_slice(),
      impulse_response_size,
    })
  }

  /// The reverb `acoustics` give a mixer rendering at `rate`, none where sound plays dry.
  pub(crate) fn for_acoustics(acoustics: &AudioAcoustics, rate: u32) -> Option<Box<Self>> {
    acoustics
      .get_steam()?
      .create_reverb(rate)
      .inspect_err(|error| log::warn!("No reverb: {error}"))
      .ok()
      .map(Box::new)
  }

  /// Renders one block of what the voices sent, decaying over `reverb_times` seconds in each band.
  pub(crate) fn apply(&mut self, reverb_times: [f32; 3], input: &[f32]) -> EarBlocks<'_> {
    let parameters: ReflectionEffectParams<Parametric> =
      ReflectionEffectParams::<Parametric>::new(reverb_times, 2, self.impulse_response_size);

    // Buffers of the block's length and two channels are always accepted.
    if let (Ok(input), Ok(mut output)) = (
      AudioBufferRef::try_from(input),
      AudioBufferMut::try_new(&mut self.out[..], 2),
    ) {
      let _ = self.effect.apply(&parameters, &input, &mut output);
    }

    let (left, right) = self.out.split_at(AudioMixer::BLOCK_FRAMES);

    EarBlocks { left, right }
  }
}
