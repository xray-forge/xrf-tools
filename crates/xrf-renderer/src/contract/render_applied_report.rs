use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_environment::RenderAppliedEnvironment;
use crate::contract::render_applied_grass::RenderAppliedGrass;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_scale::RenderScale;

/// What a viewport's frames are drawn with, as the renderer resolved what it was asked: sent as it changes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedReport {
  /// What smooths the finished frame: none where a target other than the frame is shown.
  pub antialiasing: RenderAntialiasing,
  /// How much smaller the scene is drawn than the viewport, native for anything but a level.
  pub render_scale: RenderScale,
  /// The sun's shadow, or none where no cascade is drawn.
  pub shadows: Option<RenderAppliedShadows>,
  /// The screen's ambient occlusion, or none where it is off or the scene is unlit.
  pub ambient_occlusion: Option<RenderAmbientOcclusionQuality>,
  /// The local lights, or none where they are off.
  pub lights: Option<RenderLightsSettings>,
  /// The grass planted, or none where none is.
  pub grass: Option<RenderAppliedGrass>,
  /// Whether the level's water is drawn.
  pub is_water: bool,
  /// What the weather lights the scene with now, or none for an asset viewer's rig.
  pub environment: Option<RenderAppliedEnvironment>,
  /// The lens flare whose sprite the sky draws, its `suns.ltx` section: the sun or the moon; none where the sky shows
  /// neither.
  pub sun: Option<String>,
}
