use glam::Vec3;
use xrf_renderer::{RenderFog, RenderLighting, RenderThunderSettings, RenderThunderboltStrike};

/// What a strike does to a frame: the colour it lights the sky, the sun and the fog with, the way its light travels,
/// and the bolt drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct ThunderFlash {
  /// The bolt's `color_anim` now, each channel in `[0, 1]`.
  pub color: Vec3,
  /// From the bolt towards the view, which the sun is turned to while it strikes, in renderer space.
  pub direction: Vec3,
  pub strike: RenderThunderboltStrike,
}

impl ThunderFlash {
  /// `CEffect_Thunderbolt::OnFrame` over the mix: the strike's colour added to the sky (clamped), the sun and the fog,
  /// the sun turned to come from the bolt, and the bolt drawn.
  pub fn light(&self, lighting: &RenderLighting, settings: &RenderThunderSettings) -> RenderLighting {
    let added = |base: Vec3, by: f32| base + self.color * by;
    let mut thundered: RenderLighting = lighting.clone();

    thundered.fog = lighting.fog.map(|fog| RenderFog {
      color: added(fog.color, settings.fog_color),
      ..fog
    });
    thundered.sky.color = added(lighting.sky.color, settings.sky_color).clamp(Vec3::ZERO, Vec3::ONE);
    thundered.sun_color = added(lighting.sun_color, settings.sun_color);
    thundered.sun_direction = self.direction;
    thundered.thunderbolt = Some(self.strike.clone());

    thundered
  }
}
