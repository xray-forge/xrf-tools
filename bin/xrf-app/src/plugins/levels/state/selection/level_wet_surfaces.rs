use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;

/// What rain wets surfaces with, as `CBlender_rain` binds it: the splashes' volume and the streaks down walls.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelWetSurfaces {
  /// `s_water`, a volume of rippling normals, a slice a moment.
  pub splash: LevelTextureReference,
  /// `s_waterFall`, the normals of water running down.
  pub flow: LevelTextureReference,
}

impl LevelWetSurfaces {
  /// The volume `rain_patch_normal` samples by place and time.
  pub const SPLASH_TEXTURE: &'static str = "water\\water_SBumpVolume";

  /// The map it runs down walls with.
  pub const FLOW_TEXTURE: &'static str = "water\\water_flowing_nmap";

  /// Finds both in the level's probe.
  pub fn read(source: &LevelWeatherSource) -> Self {
    Self {
      flow: source.locate(Self::FLOW_TEXTURE),
      splash: source.locate(Self::SPLASH_TEXTURE),
    }
  }
}
