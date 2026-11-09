use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A library object the scene places, `CSceneObject`: an object of `scene_object.part`, a `.object` under `$objects$`
/// at its transform, which the build merges into the level's static geometry or exports as a multi-use model.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableSceneObject {
  pub object: EditableCustomObject,
  /// The library object it places, a path under `$objects$` without its extension, such as `statics\stol\stol_4`.
  pub reference: String,
  /// `CSceneObject::m_Flags`, as stored.
  pub flags: u32,
}

impl EditableSceneObject {
  /// `SCENEOBJ_CURRENT_VERSION`, which the editor writes.
  pub const VERSION: u32 = 0x12;
}

impl EditableToolObject for EditableSceneObject {
  /// `OBJCLASS_SCENEOBJECT`.
  const CLASS_ID: i32 = 2;
  const TOOL_NAME: &'static str = "scene_object";
  const KEEPS_FLAGS: bool = true;

  /// As `CSceneObject::LoadLTX` reads it (`SceneObjectIO.cpp:18`), which takes any version.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let reference: &str = section.get_required("reference_name")?;

    if reference.is_empty() {
      return Err(XrfError::new_parsing_error(format!(
        "Scene object [{}] references no library object",
        section.name
      )));
    }

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      reference: reference.to_owned(),
      flags: EditableLevelValue::read_u32(section, "flags")?,
    })
  }

  /// As `CSceneObject::SaveLTX` writes it (`SceneObjectIO.cpp:79`), at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("reference_name", self.reference.clone());
    section.set("flags", EditableLevelValue::format_u32(self.flags));
  }
}
