use std::time::Duration;

use xrf_engine_target::XrayEngine;
use xrf_math::EPS_L;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_sound::WorldSound;
use crate::sound::emitters::sound_emitters::SoundEmitters;
use crate::sound::emitters::sound_play::SoundPlay;
use crate::sound::weather::rain_frame::RainFrame;

/// `CEffect_Rain`'s loop, `ambient\rain`: started once the rain's factor rises above `EPS_L`, stopped at once when it
/// falls back, played at the level's origin over a range, its volume the factor eased by how open the camera stands.
///
/// OpenXRay's factor is the rain's density, its volume at least a tenth of it; Monolith's is half density and half the
/// wind (only while it rains), its volume held within a tenth and one, and its range halved.
#[derive(Default)]
pub(crate) struct RainSound {
  emitter: Option<u64>,
  /// `hemi_factor`, eased toward the camera's hemi each frame.
  hemi: f32,
}

impl RainSound {
  /// Plays on a frame `delta` long to `now`, the level's clock.
  pub(crate) fn update(
    &mut self,
    emitters: &mut SoundEmitters,
    sound: Option<&WorldSound>,
    engine: XrayEngine,
    frame: RainFrame,
    delta: Duration,
    now: Duration,
  ) {
    let step: f32 = delta.as_secs_f32().clamp(0.001, 1.0);
    let hemi: f32 = frame.hemi.unwrap_or(1.0);

    self.hemi = self.hemi * (1.0 - step) + hemi * step;

    let factor: f32 = match engine {
      XrayEngine::Vanilla => frame.density,
      XrayEngine::Extended => {
        let wind: f32 = if frame.density > 0.0 {
          (frame.wind_velocity * 0.001).clamp(0.0, 1.0)
        } else {
          0.0
        };

        frame.density * 0.5 + wind * 0.5
      }
    };

    if factor < EPS_L {
      if let Some(id) = self.emitter.take() {
        emitters.stop(id);
      }

      return;
    }

    let volume: f32 = match engine {
      XrayEngine::Vanilla => factor.max(0.1) * self.hemi,
      XrayEngine::Extended => (factor * self.hemi).clamp(0.1, 1.0),
    };

    match self.emitter {
      Some(id) => emitters.set_volume(id, volume),
      None => {
        let offset: f32 = match engine {
          XrayEngine::Vanilla => 40.0,
          XrayEngine::Extended => 20.0,
        };

        self.emitter = sound.and_then(|sound| {
          emitters.play(
            engine,
            SoundPlay {
              is_looped: true,
              volume,
              range: Some([offset, offset * 2.0]),
              ..SoundPlay::new(sound, WorldSoundKind::Rain, [0.0; 3], now)
            },
          )
        });
      }
    }
  }
}
