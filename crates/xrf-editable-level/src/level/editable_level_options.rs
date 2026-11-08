use xrf_error::XrfResult;
use xrf_ini::IniSection;

use crate::level::editable_level_value::EditableLevelValue;

/// The scene's `[level_options]`, `st_LevelOptions::SaveLTX`: where the level builds to and how its lights bake.
/// Its `version` and `version_bp` are the editor's constants and stay in the section as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableLevelOptions {
  /// The level's directory under `$game_levels$`, which Build writes into.
  pub level_path: String,
  pub level_prefix: String,
  /// Text of the build options, written quoted.
  pub build_options: String,
  pub map_version: String,
  /// `st_MapUsage`'s game types the level serves.
  pub game_type: u16,
  pub light_hemi_quality: u8,
  pub light_sun_quality: u8,
}

impl EditableLevelOptions {
  pub const SECTION: &str = "level_options";

  /// `CURRENT_LEVELOP_VERSION`.
  pub const VERSION: u32 = 0x0C;

  /// `CURRENT_LEVELOP_BP_VERSION`, the build parameters' layout.
  pub const BUILD_PARAMS_VERSION: u32 = 9;

  /// # Errors
  ///
  /// Returns an error naming the field that is missing or does not parse.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      level_path: section.get_required("level_path")?.to_owned(),
      level_prefix: section.get_required("level_prefix")?.to_owned(),
      build_options: EditableLevelValue::read_quoted(section, "bop")?.to_owned(),
      map_version: section.get_required("map_version")?.to_owned(),
      game_type: section.get_parsed::<u16>("game_type")?,
      light_hemi_quality: section.get_parsed::<u8>("light_hemi_quality")?,
      light_sun_quality: section.get_parsed::<u8>("light_sun_quality")?,
    })
  }

  /// Writes every field the editor writes, its constant versions included.
  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("version", Self::VERSION.to_string());
    section.set("level_path", self.level_path.as_str());
    section.set("level_prefix", self.level_prefix.as_str());
    section.set("bop", EditableLevelValue::format_quoted(&self.build_options));
    section.set("map_version", self.map_version.as_str());
    section.set("version_bp", Self::BUILD_PARAMS_VERSION.to_string());
    section.set("light_hemi_quality", self.light_hemi_quality.to_string());
    section.set("light_sun_quality", self.light_sun_quality.to_string());
    section.set("game_type", self.game_type.to_string());
  }
}
