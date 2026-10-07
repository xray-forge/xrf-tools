use glam::Vec3;
use xrf_renderer::{RenderFog, RenderLighting, RenderThunderSettings};

use crate::weather::thunder_flash::ThunderFlash;

/// `CEffect_Thunderbolt::OnFrame` over the mix: the strike's colour added to the sky (clamped), the sun and the fog,
/// the sun turned to come from the bolt, and the bolt drawn.
pub fn to_thundered_lighting(
  lighting: &RenderLighting,
  flash: &ThunderFlash,
  settings: &RenderThunderSettings,
) -> RenderLighting {
  let added = |base: Vec3, by: f32| base + flash.color * by;
  let mut thundered: RenderLighting = lighting.clone();

  thundered.fog = lighting.fog.map(|fog| RenderFog {
    color: added(fog.color, settings.fog_color),
    ..fog
  });
  thundered.sky.color = added(lighting.sky.color, settings.sky_color).clamp(Vec3::ZERO, Vec3::ONE);
  thundered.sun_color = added(lighting.sun_color, settings.sun_color);
  thundered.sun_direction = flash.direction;
  thundered.thunderbolt = Some(flash.strike.clone());

  thundered
}
