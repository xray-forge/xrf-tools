use serde::Serialize;
use xrf_light_anim::{LightAnimItem, LightAnimKey};

use crate::data::lights::light_animator_key::LightAnimatorKey;

/// A colour animation of `lanims.xr` (`CLAItem`), which replaces the colour of every light it drives.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimatorDescription {
  pub fps: f32,
  pub frame_count: u32,
  /// By frame, the first at frame zero.
  pub keys: Vec<LightAnimatorKey>,
}

impl LightAnimatorDescription {
  /// The keys as the engine holds them once loaded, each as `RGB`: a version 0 library stores `BGR`, which the load
  /// swaps, and `CHangingLamp` swaps `CalculateBGR`'s result back.
  pub fn of(item: &LightAnimItem, is_bgr: bool) -> Self {
    let mut keys: Vec<&LightAnimKey> = item.keys.iter().collect();

    keys.sort_by_key(|key| key.frame);

    Self {
      fps: item.fps,
      frame_count: item.frame_count,
      keys: keys
        .into_iter()
        .map(|key| {
          let shifts: [u32; 3] = if is_bgr { [0, 8, 16] } else { [16, 8, 0] };

          LightAnimatorKey {
            frame: key.frame,
            color: shifts.map(|shift| ((key.color >> shift) & 0xFF) as f32),
          }
        })
        .collect(),
    }
  }
}
