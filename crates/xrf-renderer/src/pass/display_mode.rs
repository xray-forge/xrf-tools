use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_view_options::RenderViewOptions;

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

  /// The stops the physical light is lifted by before the transform: each transform's middle-grey anchor, which shows
  /// a daylit exterior's average (`ExposureHistogramUniform::DAYLIT_STOPS` under middle grey) at the display value the
  /// engine's curve shows it at, and nothing else of the engine's look, so each stays its own. None for the engine's
  /// curve and the engine's own showing.
  ///
  /// The land sits well below that average, where AgX's and PBR Neutral's toes darken it more than the engine's
  /// near-linear curve does, so a single grey cannot match them: these show a daylit land about 1.5 to 2 stops
  /// darker than the engine's curve, more at night.
  pub fn get_anchor_stops(self) -> f32 {
    match self {
      Self::Agx => -0.8,
      Self::PbrNeutral => 0.27,
      Self::AsIs | Self::Tonemapped | Self::EngineEncoded => 0.0,
    }
  }
}
