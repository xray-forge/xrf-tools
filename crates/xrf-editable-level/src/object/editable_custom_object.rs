use xrf_error::XrfResult;
use xrf_ini::IniSection;

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_object_transform::EditableObjectTransform;

/// What every scene object saves first, `CCustomObject::SaveLTX`: its flags, name and transform.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableCustomObject {
  /// `co_flags`: motion, grouping and the editor's other states, `0` for an object the editor creates. GSC's sound
  /// sources hold `2`, the bit an older editor marked visibility with (`flVisible_notused`).
  pub flags: u32,
  /// Unique within the scene, which the editor keeps by numbering names it generates (`<prefix>_0000`).
  pub name: String,
  pub transform: EditableObjectTransform,
}

impl EditableCustomObject {
  /// A named object at a transform, with no flags.
  pub fn new(name: impl Into<String>, transform: EditableObjectTransform) -> Self {
    Self {
      flags: 0,
      name: name.into(),
      transform,
    }
  }

  /// # Errors
  ///
  /// Returns an error naming the field that is missing or malformed.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      flags: EditableLevelValue::read_u32(section, "co_flags")?,
      name: section.get_required("name")?.to_owned(),
      transform: EditableObjectTransform::read_from_section(section)?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("co_flags", EditableLevelValue::format_u32(self.flags));
    section.set("name", self.name.clone());
    self.transform.write_to_section(section);
  }
}
