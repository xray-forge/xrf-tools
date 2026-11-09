use xrf_error::XrfResult;
use xrf_ini::IniSection;

use crate::level::editable_level_value::EditableLevelValue;

/// A text part's `[main]`, which every object tool writes before its objects: the count `ESceneCustomOTool::SaveLTX`
/// writes, and the version and flags the tool's own `SaveLTX` adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditablePartHeader {
  /// The tool's version, under the key its tool writes it (`version`, or the glow tool's `glow_tool_version`).
  pub version: u32,
  /// How many `[object_N]` sections follow, numbered from 0.
  pub objects_count: u32,
  /// The tool's flags, which only some tools keep: scene objects, lights, sectors, portals, spawns, glows, the AI map,
  /// detail objects and wall marks. Particles, sounds, shapes, ways and fog volumes write none.
  pub flags: Option<u32>,
}

impl EditablePartHeader {
  pub const SECTION: &str = "main";

  /// The key most tools write their version under.
  pub const VERSION_KEY: &str = "version";

  /// The key `ESceneCustomOTool::SaveLTX` writes the count under.
  pub const OBJECTS_COUNT_KEY: &str = "objects_count";

  /// Reads the header of a tool writing its version under `version_key`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the field that is missing or not an unsigned integer.
  pub fn read_from_section(section: &IniSection, version_key: &str) -> XrfResult<Self> {
    Ok(Self {
      version: section.get_parsed::<u32>(version_key)?,
      objects_count: section.get_parsed::<u32>(Self::OBJECTS_COUNT_KEY)?,
      flags: section
        .get("flags")
        .map(|_| EditableLevelValue::read_u32(section, "flags"))
        .transpose()?,
    })
  }

  /// Writes the header of a tool writing its version under `version_key`.
  pub fn write_to_section(&self, section: &mut IniSection, version_key: &str) {
    section.set(version_key, self.version.to_string());
    section.set(Self::OBJECTS_COUNT_KEY, self.objects_count.to_string());

    if let Some(flags) = self.flags {
      section.set("flags", EditableLevelValue::format_u32(flags));
    }
  }
}
