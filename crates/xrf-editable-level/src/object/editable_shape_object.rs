use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniSection};

use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_shape::EditableShape;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A shape object, `CEditShape` as the shape tool places it: an object of `shape.part`, and what a spawn point holds
/// attached to give an entity its volume (a restrictor's, a zone's).
#[derive(Clone, Debug, PartialEq)]
pub struct EditableShapeObject {
  pub object: EditableCustomObject,
  pub shape: EditableShape,
}

impl EditableShapeObject {
  /// As `CEditShape::LoadLTX` reads it from one section.
  ///
  /// # Errors
  ///
  /// Returns an error naming the field that is missing or malformed.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      shape: EditableShape::read_from_section(section)?,
    })
  }

  /// As `CEditShape::SaveLTX` writes it into one section.
  pub fn write_to_section(&self, section: &mut IniSection) {
    self.object.write_to_section(section);
    self.shape.write_to_section(section);
  }
}

impl EditableToolObject for EditableShapeObject {
  /// `OBJCLASS_SHAPE`.
  const CLASS_ID: i32 = 4;
  const TOOL_NAME: &'static str = "shape";

  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    Self::read_from_section(object.section)
  }

  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    self.write_to_section(ini.section_mut(section));
  }
}
