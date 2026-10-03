use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::state::selection::level_weather_model::LevelWeatherModel;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;

/// What rain is drawn with, as `dxRainRender` loads it: the streak's texture and the splash's model.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelRain {
  pub streak: LevelTextureReference,
  /// None where the model is not there or does not read, which draws no splashes.
  pub drop: Option<LevelWeatherModel>,
}

impl LevelRain {
  /// The texture every streak draws with, `SH_Rain.create("effects\\rain", "fx\\fx_rain")`.
  pub const STREAK_TEXTURE: &'static str = "fx\\fx_rain";

  /// The splash's model.
  pub const DROP_MODEL: &'static str = "meshes\\dm\\rain.dm";

  /// Finds the streak's texture and reads the splash's model in the level's probe.
  pub fn read(source: &LevelWeatherSource) -> Self {
    Self {
      drop: Self::read_drop(source),
      streak: source.locate(Self::STREAK_TEXTURE),
    }
  }

  fn read_drop(source: &LevelWeatherSource) -> Option<LevelWeatherModel> {
    match source.read_model(Self::DROP_MODEL) {
      Ok(model) => Some(LevelWeatherModel::of(&model, source.locate(&model.texture))),
      Err(error) => {
        log::warn!("Rain splashes are not drawn: {error}");

        None
      }
    }
  }
}
