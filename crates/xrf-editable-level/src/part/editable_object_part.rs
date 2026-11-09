use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_header::EditablePartHeader;
use crate::part::editable_part_modification::EditablePartModification;
use crate::part::editable_part_object::EditablePartObject;
use crate::part::editable_text_part::EditableTextPart;

/// An object tool's text part, typed: its scene's GUID, the tool's `[main]` and `[modif]`, and its objects in the
/// order `[object_N]` numbers them (`ESceneCustomOTool::SaveLTX`). The `[main]` version is the tool's own
/// ([`EditableToolObject::TOOL_VERSION`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableObjectPart<T> {
  pub guid: EditableLevelGuid,
  /// The tool's flags in `[main]`: present for a tool that keeps them ([`EditableToolObject::KEEPS_FLAGS`]), and for
  /// another kept as its part stored them, which the editor ignores.
  pub flags: Option<u32>,
  pub modification: EditablePartModification,
  pub objects: Vec<T>,
}

impl<T: EditableToolObject> EditableObjectPart<T> {
  /// A part of no modification holding `objects`, its flags clear where the tool keeps any.
  pub fn new(guid: EditableLevelGuid, objects: Vec<T>) -> Self {
    Self {
      guid,
      flags: T::KEEPS_FLAGS.then_some(0),
      modification: EditablePartModification::default(),
      objects,
    }
  }

  /// # Errors
  ///
  /// Returns an error, as the tool's `LoadLTX` refuses the part, when it has no `[main]` or `[modif]`, its version is
  /// not the tool's, it keeps no flags where the tool requires them, an object is of another tool's class, or a field
  /// an object needs is missing or malformed.
  pub fn read_from_text_part(part: &EditableTextPart) -> XrfResult<Self> {
    let header: EditablePartHeader = part
      .get_header()?
      .ok_or_else(|| XrfError::new_parsing_error(format!("Part has no section [{}]", EditablePartHeader::SECTION)))?;

    if header.version != T::TOOL_VERSION || (T::KEEPS_FLAGS && header.flags.is_none()) {
      return Err(XrfError::new_parsing_error(format!(
        "Part of class {} holds tool version {}{}, where the tool reads version {}{}",
        T::CLASS_ID,
        header.version,
        if header.flags.is_some() { " with flags" } else { "" },
        T::TOOL_VERSION,
        if T::KEEPS_FLAGS { " with flags" } else { "" }
      )));
    }

    let objects: Vec<T> = part
      .get_objects()?
      .iter()
      .map(|object| {
        let class_id: i32 = object.get_class_id()?;

        if class_id != T::CLASS_ID {
          return Err(XrfError::new_parsing_error(format!(
            "Object [{}] is of class {class_id}, not the tool's {}",
            object.section.name,
            T::CLASS_ID
          )));
        }

        T::read_from_object(object)
      })
      .collect::<XrfResult<Vec<T>>>()?;

    Ok(Self {
      guid: part.get_guid()?,
      flags: header.flags,
      modification: EditablePartModification::read_from_section(
        part.ini.section_required(EditablePartModification::SECTION)?,
      )?,
      objects,
    })
  }

  /// The text part as `EScene::SaveToolLTX` writes it.
  ///
  /// # Errors
  ///
  /// Returns an error when the part holds more objects than `[main]` can count.
  pub fn to_text_part(&self) -> XrfResult<EditableTextPart> {
    let mut ini: IniFile = IniFile::default();

    EditablePartHeader {
      version: T::TOOL_VERSION,
      objects_count: u32::try_from(self.objects.len())
        .map_err(|_| XrfError::new_invalid_error(format!("Part holds {} objects", self.objects.len())))?,
      flags: self.flags,
    }
    .write_to_section(ini.section_mut(EditablePartHeader::SECTION));
    self
      .modification
      .write_to_section(ini.section_mut(EditablePartModification::SECTION));

    for (index, object) in self.objects.iter().enumerate() {
      let name: String = EditablePartObject::get_section_name(index);
      let section: &mut IniSection = ini.section_mut(&name);

      section.set("clsid", T::CLASS_ID.to_string());
      object.write_to_ini(&mut ini, &name);
    }

    let mut part: EditableTextPart = EditableTextPart { ini };

    part.set_guid(&self.guid);

    Ok(part)
  }
}
