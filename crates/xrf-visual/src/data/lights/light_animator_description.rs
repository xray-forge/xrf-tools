use serde::Serialize;
use xrf_light_anim::LightAnimItem;

use crate::data::lights::light_animator_key::LightAnimatorKey;

/// A colour animation of `lanims.xr` (`CLAItem`), which replaces the colour of every light it drives.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimatorDescription {
  pub fps: f32,
  pub frame_count: u32,
  /// By frame, one key a frame; shipped libraries key frames before zero too.
  pub keys: Vec<LightAnimatorKey>,
}

impl LightAnimatorDescription {
  /// The keys as the engine holds them once loaded (`LightAnimItem::to_loaded_keys`), each as `RGB`: `CHangingLamp`
  /// swaps `CalculateBGR`'s result back.
  pub fn of(item: &LightAnimItem, is_bgr: bool) -> Self {
    Self {
      fps: item.fps,
      frame_count: item.frame_count,
      keys: item
        .to_loaded_keys(is_bgr)
        .into_iter()
        .map(|key| LightAnimatorKey {
          frame: key.frame,
          color: [key.get_red(), key.get_green(), key.get_blue()].map(f32::from),
        })
        .collect(),
    }
  }
}
