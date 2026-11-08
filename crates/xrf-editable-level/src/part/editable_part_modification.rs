use xrf_error::XrfResult;
use xrf_ini::IniSection;

/// A text part's `[modif]`: who last changed the tool and when (`ESceneToolBase::SaveLTX`). Every scene GSC shipped
/// leaves both empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditablePartModification {
  pub name: String,
  /// Unix seconds, `0` for never.
  pub time: u32,
}

impl EditablePartModification {
  pub const SECTION: &str = "modif";

  /// # Errors
  ///
  /// Returns an error naming the field that is missing or, for the time, not an unsigned integer.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      name: section.get_required("name")?.to_owned(),
      time: section.get_parsed::<u32>("time")?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("name", self.name.clone());
    section.set("time", self.time.to_string());
  }
}
