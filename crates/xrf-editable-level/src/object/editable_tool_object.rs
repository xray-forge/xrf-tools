use xrf_error::XrfResult;
use xrf_ini::IniFile;

use crate::part::editable_part_object::EditablePartObject;

/// An object one of the Level Editor's object tools keeps, typed: what it reads from its `[object_N]` section, and the
/// sections named after it, and writes back as its `SaveLTX` does.
pub trait EditableToolObject: Sized {
  /// The editor's `ObjClassID` of the tool's objects, written as each object's `clsid`.
  const CLASS_ID: i32;

  /// # Errors
  ///
  /// Returns an error when a field the object needs is missing or malformed.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self>;

  /// Writes the object's fields under `section`, and any section named after it, `clsid` aside.
  fn write_to_ini(&self, ini: &mut IniFile, section: &str);
}
