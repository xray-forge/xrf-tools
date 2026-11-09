use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_level_part::EditableLevelPart;
use crate::part::editable_part_header::EditablePartHeader;
use crate::part::editable_part_modification::EditablePartModification;
use crate::part::editable_part_object::EditablePartObject;
use crate::part::editable_text_part::EditableTextPart;
use crate::scene::editable_scene_tool::EditableSceneTool;

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
  /// What the tool itself saves beyond its header, kept as stored: further `[main]` keys and sections of its own, such
  /// as the light tool's `sun_shadow_dir`, `lcontrol_last_idx` and `[lcontrols]` (`ESceneLightTool::SaveLTX`). Empty for
  /// a tool saving nothing more.
  pub tool: IniFile,
}

impl<T: EditableToolObject> EditableObjectPart<T> {
  /// A part of no modification holding `objects`, its flags clear where the tool keeps any, and no tool data.
  pub fn new(guid: EditableLevelGuid, objects: Vec<T>) -> Self {
    Self {
      guid,
      flags: T::KEEPS_FLAGS.then_some(0),
      modification: EditablePartModification::default(),
      objects,
      tool: IniFile::default(),
    }
  }

  /// The part's sections and `[main]` keys that neither the header, `[guid]`, `[modif]` nor an object holds.
  fn read_tool_data(part: &EditableTextPart) -> IniFile {
    let mut tool: IniFile = IniFile::default();

    for section in &part.ini.sections {
      match section.name.as_str() {
        EditableLevelGuid::SECTION | EditablePartModification::SECTION => {}
        EditablePartHeader::SECTION => {
          for field in section.fields.iter().filter(|field| {
            ![
              T::TOOL_VERSION_KEY,
              EditablePartHeader::OBJECTS_COUNT_KEY,
              EditablePartHeader::FLAGS_KEY,
            ]
            .contains(&field.key.as_str())
          }) {
            tool
              .section_mut(EditablePartHeader::SECTION)
              .set(field.key.clone(), field.value.clone());
          }
        }
        name if name.starts_with(EditablePartObject::SECTION_PREFIX) => {}
        _ => tool.sections.push(section.clone()),
      }
    }

    tool
  }

  /// # Errors
  ///
  /// Returns an error, as the tool's `LoadLTX` refuses the part, when it has no `[main]` or `[modif]`, its version is
  /// not the tool's, it keeps no flags where the tool requires them, an object is of another tool's class, or a field
  /// an object needs is missing or malformed.
  pub fn read_from_text_part(part: &EditableTextPart) -> XrfResult<Self> {
    let header: EditablePartHeader = part
      .get_header(T::TOOL_VERSION_KEY)?
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
      tool: Self::read_tool_data(part),
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
    .write_to_section(ini.section_mut(EditablePartHeader::SECTION), T::TOOL_VERSION_KEY);
    self
      .modification
      .write_to_section(ini.section_mut(EditablePartModification::SECTION));

    for section in &self.tool.sections {
      let written: &mut IniSection = ini.section_mut(&section.name);

      for field in &section.fields {
        written.set(field.key.clone(), field.value.clone());
      }
    }

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

  /// The part a scene's tool holds, read as [`Self::read_from_text_part`] reads it.
  ///
  /// # Errors
  ///
  /// Returns an error when the tool is not this one ([`EditableToolObject::TOOL_NAME`]), holds other than one part, or
  /// holds a binary part, or as [`Self::read_from_text_part`] does.
  pub fn read_from_scene_tool(tool: &EditableSceneTool) -> XrfResult<Self> {
    match tool.parts.as_slice() {
      [EditableLevelPart::Text(part)] if tool.name == T::TOOL_NAME => Self::read_from_text_part(part),
      _ => Err(XrfError::new_parsing_error(format!(
        "Scene tool '{}' of {} parts is not the one text part of tool '{}'",
        tool.name,
        tool.parts.len(),
        T::TOOL_NAME
      ))),
    }
  }

  /// The tool's place in a scene: its one text part under its name ([`EditableToolObject::TOOL_NAME`]).
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::to_text_part`] does.
  pub fn to_scene_tool(&self) -> XrfResult<EditableSceneTool> {
    Ok(EditableSceneTool {
      name: T::TOOL_NAME.to_owned(),
      parts: vec![EditableLevelPart::Text(self.to_text_part()?)],
    })
  }
}
