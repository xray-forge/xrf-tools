use xrf_error::XrfResult;
use xrf_ini::IniSection;

/// A text part's `[main]`, which every object tool writes before its objects (`ESceneCustomOTools::SaveLTX`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditablePartHeader {
  pub version: u32,
  /// How many `[object_N]` sections follow, numbered from 0.
  pub objects_count: u32,
  /// The tool's flags, which only some tools keep: scene objects, lights, sectors, portals, spawns, glows, the AI map,
  /// detail objects and wall marks. Particles, sounds, shapes, ways and fog volumes write none.
  pub flags: Option<u32>,
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
      flags: section
        .get("flags")
        .map(|_| section.get_parsed::<u32>("flags"))
        .transpose()?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("version", self.version.to_string());
    section.set("objects_count", self.objects_count.to_string());

    if let Some(flags) = self.flags {
      section.set("flags", flags.to_string());
    }
  }
}
