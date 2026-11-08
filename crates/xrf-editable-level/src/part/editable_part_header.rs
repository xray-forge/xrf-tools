use xrf_error::XrfResult;
use xrf_ini::IniSection;

/// A text part's `[main]`, which every object tool writes before its objects (`ESceneCustomOTools::SaveLTX`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditablePartHeader {
  pub version: u32,
  /// How many `[object_N]` sections follow, numbered from 0.
  pub objects_count: u32,
  pub flags: u32,
}

impl EditablePartHeader {
  pub const SECTION: &str = "main";

  /// # Errors
  ///
  /// Returns an error naming the field that is missing or not an unsigned integer.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      version: section.get_parsed::<u32>("version")?,
      objects_count: section.get_parsed::<u32>("objects_count")?,
      flags: section.get_parsed::<u32>("flags")?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("version", self.version.to_string());
    section.set("objects_count", self.objects_count.to_string());
    section.set("flags", self.flags.to_string());
  }
}
