use serde::Serialize;
use xrf_chunk::XRayByteOrder;
use xrf_level::DetailModel;
use xrf_vfs::XrayResolution;

use crate::plugins::levels::state::selection::level_rain_drop::LevelRainDrop;
use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;

/// What rain is drawn with, as `dxRainRender` loads it: the streak's texture and the splash's model.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelRain {
  pub streak: LevelTextureReference,
  /// None where the model is not there or does not read, which draws no splashes.
  pub drop: Option<LevelRainDrop>,
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

  fn read_drop(source: &LevelWeatherSource) -> Option<LevelRainDrop> {
    let read = source
      .probe
      .find(Self::DROP_MODEL)
      .map_err(|error| error.to_string())
      .and_then(|resolution: XrayResolution| {
        let asset = resolution
          .get_asset()
          .ok_or_else(|| format!("'{}' is not in the mounted roots", Self::DROP_MODEL))?;

        source.probe.read_asset_bytes(asset).map_err(|error| error.to_string())
      })
      .and_then(|bytes| DetailModel::read_from_bytes::<XRayByteOrder>(bytes).map_err(|error| error.to_string()));

    match read {
      Ok(model) => Some(LevelRainDrop::of(&model, source.locate(&model.texture))),
      Err(error) => {
        log::warn!("Rain splashes are not drawn: {error}");

        None
      }
    }
  }
}
