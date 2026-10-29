use serde::{Deserialize, Serialize};

/// How the lamps fall off at the anchored light level: by the windowed inverse square, as light falls off in the world,
/// or by the engine's own curve, as Unreal keeps an exponent falloff for lights not authored physically. The authored
/// levels keep the engine's, which their summed decode makes a pool of; the physical levels the inverse square.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderLightFalloff {
  /// The windowed inverse square from the lamp's bulb, at the intensity that gives as much light within its reach as
  /// the engine's falloff: a pool about the bulb, dimmer towards the reach.
  #[default]
  Physical,
  /// The engine's `1 - d²/r²`: the game's flat lamp-lit rooms.
  Engine,
}
