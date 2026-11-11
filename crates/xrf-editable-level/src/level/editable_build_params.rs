use xrf_error::XrfResult;
use xrf_ini::IniSection;

use crate::level::editable_level_value::EditableLevelValue;

/// The scene's `[build_params]`, `b_params::SaveLTX` (`LevelEditor/Engine/communicate.h`): what the compilers weld,
/// smooth and bake with, and the reserved fields, which the Build copies into `build.prj` as they are.
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
  /// `m_quality`'s padding as `b_params` holds it, whatever memory left there: GSC's `labx8` holds 29795.
  pub light_quality_reserved: u16,
  /// `reserved_0` to `reserved_5`, likewise.
  pub reserved: [f32; Self::RESERVED_FLOATS],
}

impl EditableBuildParams {
  pub const SECTION: &str = "build_params";

  /// The reserved floats `b_params` saves and loads, `reserved_0` to `reserved_5`.
  pub const RESERVED_FLOATS: usize = 6;

  /// `b_params::Init` then `setHighQuality`, what `st_LevelOptions::Reset` gives a new level, its reserved fields zero.
  pub const DEFAULT: Self = Self {
    smooth_angle: 75.0,
    weld_distance: 0.005,
    light_pixel_per_meter: 10.0,
    light_jitter_samples: 9,
    light_rms_zero: 4,
    light_rms: 4,
    light_quality: 1,
    light_quality_reserved: 0,
    reserved: [0.0; Self::RESERVED_FLOATS],
  };

  /// The reserved fields read as zero where the section lacks them, as an early decompiled scene does.
  ///
  /// # Errors
  ///
  /// Returns an error naming the field that is missing or does not parse.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    let mut reserved: [f32; Self::RESERVED_FLOATS] = [0.0; Self::RESERVED_FLOATS];

    for (index, value) in reserved.iter_mut().enumerate() {
      let key: String = format!("reserved_{index}");

      if section.get(&key).is_some() {
        *value = section.get_parsed::<f32>(&key)?;
      }
    }

    Ok(Self {
      smooth_angle: section.get_parsed::<f32>("smooth_angle")?,
      weld_distance: section.get_parsed::<f32>("weld_distance")?,
      light_pixel_per_meter: section.get_parsed::<f32>("light_pixel_per_meter")?,
      light_jitter_samples: section.get_parsed::<u32>("light_jitter_samples")?,
      light_rms_zero: section.get_parsed::<u32>("light_rms_zero")?,
      light_rms: section.get_parsed::<u32>("light_rms")?,
      light_quality: section.get_parsed::<u16>("light_quality")?,
      light_quality_reserved: match section.get("light_quality_reserved") {
        Some(_) => section.get_parsed::<u16>("light_quality_reserved")?,
        None => 0,
      },
      reserved,
    })
  }

  /// Writes every field `b_params::LoadLTX` reads: the SDK's editor refuses a scene lacking a reserved one ("Can't find
  /// variable reserved_1 in [build_params]"), and every GSC scene of SDK 0.7 holds the quality's and all six floats.
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
    section.set("light_quality_reserved", self.light_quality_reserved.to_string());

    for (index, value) in self.reserved.iter().enumerate() {
      section.set(format!("reserved_{index}"), EditableLevelValue::format_f32(*value));
    }
  }
}
