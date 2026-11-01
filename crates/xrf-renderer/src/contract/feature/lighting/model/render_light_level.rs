use serde::{Deserialize, Serialize};

/// How bright the physical light's sources stand against each other and the day: as the weathers and the lamps are
/// authored, or brought to the real world's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderLightLevel {
  /// The weathers' and the lamps' own levels in the physical light's units: a night as bright as the game authors it,
  /// a lamp a thousand times a real one.
  #[default]
  Authored,
  /// The authored levels as additive linear light: each light anchored, per channel, so open shade shows the engine's
  /// decoded hemisphere and ambient, full sun the engine's decoded sum, and a lamp its decoded colour over the ambient,
  /// the lights adding linearly between, where the authored levels decode their sum once as the engine's display does.
  Anchored,
  /// The real world's: the weather's light dimmed with its sky towards a moonlit night of a fraction of a lux, the lamps
  /// and the self-lit surfaces brought to real lamps', and the exposure held from showing the dark more than a few stops
  /// under middle grey.
  Physical,
}
