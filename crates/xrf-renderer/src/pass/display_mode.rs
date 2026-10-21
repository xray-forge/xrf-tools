use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::display_calibration::DisplayCalibration;

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

  /// How the physical light's transform is calibrated to the engine's showing over the game's scenes; the engine's own
  /// showing calibrates nothing.
  pub fn get_calibration(self) -> DisplayCalibration {
    match self {
      Self::EngineEncoded => DisplayCalibration {
        stops: -0.3,
        saturation: 1.23,
        kept_share: 0.58,
        picture_stops: 0.39,
        picture_adaptation: 1.1,
      },
      Self::Agx => DisplayCalibration {
        stops: 0.13,
        saturation: 1.15,
        kept_share: 0.43,
        picture_stops: -0.8,
        picture_adaptation: 1.4,
      },
      Self::PbrNeutral => DisplayCalibration {
        stops: 1.8,
        saturation: 0.55,
        kept_share: 0.59,
        picture_stops: -1.54,
        picture_adaptation: 1.67,
      },
      Self::AsIs | Self::Tonemapped => DisplayCalibration::NONE,
    }
  }
}
