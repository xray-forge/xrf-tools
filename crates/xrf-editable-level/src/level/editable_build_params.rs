use xrf_error::XrfResult;
use xrf_ini::IniSection;

use crate::level::editable_level_value::EditableLevelValue;

/// The scene's `[build_params]`, `b_params::SaveLTX` (`LevelEditor/Engine/communicate.h`): what the compilers weld,
/// smooth and bake with. The reserved fields stay in the section as written.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableBuildParams {
  /// Degrees between faces' normals that still smooth into one.
  pub smooth_angle: f32,
  /// Metres within which xrLC welds vertices; the editor's default is 0.005.
  pub weld_distance: f32,
  pub light_pixel_per_meter: f32,
  pub light_jitter_samples: u32,
  pub light_rms_zero: u32,
  pub light_rms: u32,
  pub light_quality: u16,
}

impl EditableBuildParams {
  pub const SECTION: &str = "build_params";

  /// `b_params::Init` then `setHighQuality`, what `st_LevelOptions::Reset` gives a new level.
  pub const DEFAULT: Self = Self {
    smooth_angle: 75.0,
    weld_distance: 0.005,
    light_pixel_per_meter: 10.0,
    light_jitter_samples: 9,
    light_rms_zero: 4,
    light_rms: 4,
    light_quality: 1,
  };

  /// # Errors
  ///
  /// Returns an error naming the field that is missing or does not parse.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      smooth_angle: section.get_parsed::<f32>("smooth_angle")?,
      weld_distance: section.get_parsed::<f32>("weld_distance")?,
      light_pixel_per_meter: section.get_parsed::<f32>("light_pixel_per_meter")?,
      light_jitter_samples: section.get_parsed::<u32>("light_jitter_samples")?,
      light_rms_zero: section.get_parsed::<u32>("light_rms_zero")?,
      light_rms: section.get_parsed::<u32>("light_rms")?,
      light_quality: section.get_parsed::<u16>("light_quality")?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("smooth_angle", EditableLevelValue::format_f32(self.smooth_angle));
    section.set("weld_distance", EditableLevelValue::format_f32(self.weld_distance));
    section.set(
      "light_pixel_per_meter",
      EditableLevelValue::format_f32(self.light_pixel_per_meter),
    );
    section.set("light_jitter_samples", self.light_jitter_samples.to_string());
    section.set("light_rms_zero", self.light_rms_zero.to_string());
    section.set("light_rms", self.light_rms.to_string());
    section.set("light_quality", self.light_quality.to_string());
  }
}
