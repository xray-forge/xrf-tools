use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};
use xrf_math::{Aabb, Matrix4x4};

use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_shape::{EditableShape, EditableShapeElement};
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A volume of reverb, `ESoundEnvironment`: an object of `sound_env.part`, a shape whose first box separates an inner
/// environment from an outer one, each a section of the sound environment library.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableSoundEnvironmentObject {
  pub object: EditableCustomObject,
  pub shape: EditableShape,
  /// `env_inner`, the environment inside the box.
  pub inner: String,
  /// `env_outer`, the environment outside it.
  pub outer: String,
}

impl EditableSoundEnvironmentObject {
  /// `SOUND_ENV_VERSION`, written over the shape's own `version` and the only one `LoadLTX` reads.
  pub const VERSION: u32 = 0x12;

  /// The world-space box splitting the volume's environments, `ESoundEnvironment::get_box`. None when the first element
  /// is not a box, which the editor asserts on.
  pub fn get_box(&self) -> Option<Matrix4x4> {
    match self.shape.elements.first()? {
      EditableShapeElement::Box(matrix) => Some(self.object.transform.to_matrix().multiply(&Matrix4x4::from(matrix))),
      EditableShapeElement::Sphere { .. } => None,
    }
  }

  /// The world-space box holding the volume, `CEditShape::GetBox`. None for a shape of no elements, which the editor
  /// asserts on.
  pub fn get_bounds(&self) -> Option<Aabb> {
    Some(self.shape.get_bounds()?.transform(&self.object.transform.to_matrix()))
  }

  /// Whether the export writes the volume: its environments differ and neither is empty.
  pub fn is_exported(&self) -> bool {
    self.inner != self.outer && !self.inner.is_empty() && !self.outer.is_empty()
  }
}

impl EditableToolObject for EditableSoundEnvironmentObject {
  /// `OBJCLASS_SOUND_ENV`.
  const CLASS_ID: i32 = 10;
  const TOOL_NAME: &'static str = "sound_env";

  /// As `ESoundEnvironment::LoadLTX` reads it, refusing another version as it does.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Sound environment [{}] is of version {version}, not {}",
        section.name,
        Self::VERSION
      )));
    }

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      shape: EditableShape::read_from_section(section)?,
      inner: section.get_required("env_inner")?.to_owned(),
      outer: section.get_required("env_outer")?.to_owned(),
    })
  }

  /// As `ESoundEnvironment::SaveLTX` writes it, its [`Self::VERSION`] over the shape's.
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    self.shape.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("env_inner", self.inner.clone());
    section.set("env_outer", self.outer.clone());
  }
}
