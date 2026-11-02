//! Editing a post-process effect's keys as the SDK's post-process editor does, through `CPostProcessParam`.

use xrf_animation_envelope::AnimationEnvelope;

use crate::{PpeColorChannel, PpeColorMap, PpeFile, PpeParameter};

impl PpeFile {
  /// How near a time must be to a key's for the editor to take it for that key, `FindKey`'s `0.01` in
  /// `CPostProcessParam`'s `add_value`, `update_value` and `get_value`.
  pub const KEY_FIND_EPSILON: f32 = 0.01;

  /// The envelope a parameter's channel is kept in: the colour's channel for a colour, and the value's one envelope for
  /// a value whatever channel is asked for, as the engine ignores the index there. `None` for the colour map's
  /// influence of an effect that stores no colour map.
  pub fn get_envelope(&self, parameter: PpeParameter, channel: PpeColorChannel) -> Option<&AnimationEnvelope> {
    Some(match parameter {
      PpeParameter::BaseColor => self.base_color.get_channel(channel),
      PpeParameter::AddColor => self.add_color.get_channel(channel),
      PpeParameter::GrayColor => self.gray_color.get_channel(channel),
      PpeParameter::GrayValue => &self.gray_value,
      PpeParameter::Blur => &self.blur,
      PpeParameter::DualityHorizontal => &self.duality_horizontal,
      PpeParameter::DualityVertical => &self.duality_vertical,
      PpeParameter::NoiseIntensity => &self.noise_intensity,
      PpeParameter::NoiseGranularity => &self.noise_granularity,
      PpeParameter::NoiseFps => &self.noise_fps,
      PpeParameter::ColorMapInfluence => &self.color_map.as_ref()?.influence,
    })
  }

  /// The envelope as [`Self::get_envelope`] finds it, giving the effect a colour map, and so version 2, where the
  /// influence is asked for and it stores none: the engine always holds one.
  pub fn get_envelope_mut(&mut self, parameter: PpeParameter, channel: PpeColorChannel) -> &mut AnimationEnvelope {
    match parameter {
      PpeParameter::BaseColor => self.base_color.get_channel_mut(channel),
      PpeParameter::AddColor => self.add_color.get_channel_mut(channel),
      PpeParameter::GrayColor => self.gray_color.get_channel_mut(channel),
      PpeParameter::GrayValue => &mut self.gray_value,
      PpeParameter::Blur => &mut self.blur,
      PpeParameter::DualityHorizontal => &mut self.duality_horizontal,
      PpeParameter::DualityVertical => &mut self.duality_vertical,
      PpeParameter::NoiseIntensity => &mut self.noise_intensity,
      PpeParameter::NoiseGranularity => &mut self.noise_granularity,
      PpeParameter::NoiseFps => &mut self.noise_fps,
      PpeParameter::ColorMapInfluence => &mut self.color_map.get_or_insert_with(PpeColorMap::default).influence,
    }
  }

  /// Sets a key, `CPostProcessParam::add_value`: the envelope's key at `time` takes the value, or a new one is inserted
  /// there, and either way loses its tension, continuity and bias.
  pub fn insert_key(&mut self, parameter: PpeParameter, channel: PpeColorChannel, time: f32, value: f32) {
    let envelope: &mut AnimationEnvelope = self.get_envelope_mut(parameter, channel);

    envelope.insert_key(time, value);

    if let Some(index) = envelope.find_key(time, Self::KEY_FIND_EPSILON) {
      let interpolation = &mut envelope.keys[index].interpolation;

      interpolation.tension = 0.0;
      interpolation.continuity = 0.0;
      interpolation.bias = 0.0;
    }
  }

  /// Takes out the key at `time`, `CPostProcessParam::delete_value`: from all three channels of a colour.
  pub fn delete_key(&mut self, parameter: PpeParameter, time: f32) {
    if parameter.is_color() {
      for channel in PpeColorChannel::ALL {
        self.get_envelope_mut(parameter, channel).delete_key(time);
      }
    } else if parameter != PpeParameter::ColorMapInfluence || self.color_map.is_some() {
      self.get_envelope_mut(parameter, PpeColorChannel::Red).delete_key(time);
    }
  }

  /// The times a parameter is keyed at, as the editor's timeline lists them (`get_key_time`): a colour's red channel's,
  /// whatever the other two hold.
  pub fn get_key_times(&self, parameter: PpeParameter) -> Vec<f32> {
    self
      .get_envelope(parameter, PpeColorChannel::Red)
      .map(|envelope| envelope.keys.iter().map(|key| key.time).collect())
      .unwrap_or_default()
  }
}
