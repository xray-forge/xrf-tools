use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_header::EditablePartHeader;
use crate::part::editable_part_modification::EditablePartModification;
use crate::part::editable_part_object::EditablePartObject;
use crate::part::editable_text_part::EditableTextPart;

/// An object tool's text part, typed: its scene's GUID, the tool's `[main]` and `[modif]`, and its objects in the
/// order `[object_N]` numbers them (`ESceneCustomOTool::SaveLTX`).
#[derive(Clone, Debug, PartialEq)]
pub struct EditableObjectPart<T> {
  pub guid: EditableLevelGuid,
  /// The tool's version in `[main]`, such as `PS_TOOLS_VERSION`.
  pub version: u32,
  /// The tool's flags in `[main]`, for the tools that keep them.
  pub flags: Option<u32>,
  pub modification: EditablePartModification,
  pub objects: Vec<T>,
}

impl<T: EditableToolObject> EditableObjectPart<T> {
  /// A part of no modification, holding `objects`.
  pub fn new(guid: EditableLevelGuid, version: u32, flags: Option<u32>, objects: Vec<T>) -> Self {
    Self {
      guid,
      version,
      flags,
      modification: EditablePartModification::default(),
      objects,
    }
  }

  /// # Errors
  ///
  /// Returns an error when the part has no `[main]` or `[modif]`, an object is of another tool's class, or a field an
  /// object needs is missing or malformed.
  pub fn read_from_text_part(part: &EditableTextPart) -> XrfResult<Self> {
    let header: EditablePartHeader = part
      .get_header()?
      .ok_or_else(|| XrfError::new_parsing_error(format!("Part has no section [{}]", EditablePartHeader::SECTION)))?;
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
      version: header.version,
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
      version: self.version,
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
