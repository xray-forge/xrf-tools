use xrf_error::XrfResult;
use xrf_ini::IniSection;
use xrf_math::Vector3d;

use crate::level::editable_level_value::EditableLevelValue;

/// One object of a text part: its `[object_N]` section and the sections named after it, such as a spawn's
/// `[object_N_spawndata]` or an attached shape's `[object_N_attached_0]`. What every object writes first is
/// `CCustomObject::SaveLTX`'s: its class, name and transform.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditablePartObject<'a> {
  pub index: usize,
  pub section: &'a IniSection,
  /// The sections whose names start with the object's and `_`, in the order the file holds them.
  pub children: Vec<&'a IniSection>,
}

impl EditablePartObject<'_> {
  /// What every object's section name starts with, its index following (`ESceneCustomOTools::SaveLTX`).
  pub const SECTION_PREFIX: &str = "object_";

  /// The name of the section an object of that index is written under.
  pub fn get_section_name(index: usize) -> String {
    format!("{}{index}", Self::SECTION_PREFIX)
  }

  /// The editor's `ObjClassID` of the object, `clsid`.
  ///
  /// # Errors
  ///
  /// Returns an error when the field is missing or not an integer.
  pub fn get_class_id(&self) -> XrfResult<i32> {
    self.section.get_parsed::<i32>("clsid")
  }

  /// # Errors
  ///
  /// Returns an error when the field is missing.
  pub fn get_name(&self) -> XrfResult<&str> {
    self.section.get_required("name")
  }

  /// # Errors
  ///
  /// Returns an error when the field is missing or not three floats.
  pub fn get_position(&self) -> XrfResult<Vector3d> {
    EditableLevelValue::read_vector3(self.section, "position")
  }

  /// Heading, pitch and bank, in radians.
  ///
  /// # Errors
  ///
  /// Returns an error when the field is missing or not three floats.
  pub fn get_rotation(&self) -> XrfResult<Vector3d> {
    EditableLevelValue::read_vector3(self.section, "rotation")
  }

  /// # Errors
  ///
  /// Returns an error when the field is missing or not three floats.
  pub fn get_scale(&self) -> XrfResult<Vector3d> {
    EditableLevelValue::read_vector3(self.section, "scale")
  }

  /// A section named after the object, such as `spawndata` for `[object_N_spawndata]`.
  pub fn get_child(&self, suffix: &str) -> Option<&IniSection> {
    let name: String = format!("{}_{suffix}", self.section.name);

    self.children.iter().find(|child| child.name == name).copied()
  }
}
