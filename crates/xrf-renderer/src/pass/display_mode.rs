use glam::Vec3;

use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::display_curves::{agx, get_graded, get_key_match, pbr_neutral};

/// How the window pass shows a viewport's scene, numbered as `common/display_transform.wgsl` numbers it: as it is (the
/// engine's passes tonemapped it), by the engine's curve (linear light at the engine's display values), or, where the
/// light is physical, by a display transform and encoded for the display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisplayMode {
  AsIs = 0,
  Tonemapped = 1,
  EngineEncoded = 2,
  Agx = 3,
  PbrNeutral = 4,
}

impl DisplayMode {
  /// The land's colour a transform is anchored on, a warm earth: PBR Neutral's toe takes its offset from a colour's
  /// weakest channel, so a grey shows the land's brightness through it as no land colour does.
  pub const ANCHOR_HUE: Vec3 = Vec3::new(1.0, 0.75, 0.45);
  /// The keys an anchor is solved within: the least the engine's curve shows above the display's first level,
  /// `(1/255)^2.2`, under which there is nothing shown to match and AgX's black would lift the frame without end, and
  /// the display's white, past which the engine's curve shows what AgX and PBR Neutral, held under white, reach only at
  /// their bounds.
  const ANCHOR_KEYS: (f32, f32) = (5.07e-6, 1.0);
  /// The most a transform's anchor lifts or lowers the light by, in stops.
  const ANCHOR_BOUNDS: (f32, f32) = (-6.0, 8.0);

  /// The mode a view's options ask for.
  pub fn of(options: &RenderViewOptions) -> Self {
    if !options.is_scene_linear() {
      return Self::AsIs;
    }

    if !options.is_light_physical() {
      return Self::Tonemapped;
    }

    match options.features.display_transform {
      RenderDisplayTransform::Engine => Self::EngineEncoded,
      RenderDisplayTransform::Agx => Self::Agx,
      RenderDisplayTransform::PbrNeutral => Self::PbrNeutral,
    }
  }

  /// The number the shaders tell it by.
  pub fn get_index(self) -> u32 {
    self as u32
  }

  /// The stops the physical light is lifted by after its grading, before the transform: each transform's anchor, which
  /// shows the land's colour (`ANCHOR_HUE`) at the frame's exposed `key` (`ExposureHead::key`), graded by `contrast`,
  /// at the display value the engine's curve shows it at, and nothing else of the engine's look, so each keeps its own
  /// contrast about it. None for the engine's curve and the engine's own showing.
  pub fn get_anchor_stops(self, key: f32, contrast: f32) -> f32 {
    let (least, most): (f32, f32) = Self::ANCHOR_KEYS;
    let shown: f32 = get_graded(key, contrast).clamp(least, most);

    match self {
      Self::Agx => get_key_match(agx, (shown, Self::ANCHOR_HUE), Self::ANCHOR_BOUNDS),
      Self::PbrNeutral => get_key_match(pbr_neutral, (shown, Self::ANCHOR_HUE), Self::ANCHOR_BOUNDS),
      Self::AsIs | Self::Tonemapped | Self::EngineEncoded => 0.0,
    }
  }
}
