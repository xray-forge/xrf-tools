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

  /// The stops the physical light's transform lifts the frame by before it: each maps middle grey its own way, and
  /// these bring each to the engine's brightness over the game's scenes. None for the engine's own showing.
  pub fn get_exposure_stops(self) -> f32 {
    match self {
      Self::EngineEncoded => -0.35,
      Self::Agx => 0.0,
      Self::PbrNeutral => 0.89,
      Self::AsIs | Self::Tonemapped => 0.0,
    }
  }

  /// The saturation the physical light's transform grades the frame at before it: each keeps chroma its own way, and
  /// these bring each to the engine's saturation over the game's scenes, which its curve tamed as it compressed them.
  /// One for the engine's own showing.
  pub fn get_saturation(self) -> f32 {
    match self {
      Self::EngineEncoded => 1.13,
      Self::Agx => 1.01,
      Self::PbrNeutral => 0.5,
      Self::AsIs | Self::Tonemapped => 1.0,
    }
  }
}
