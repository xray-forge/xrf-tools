use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniFormat, IniSection};

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_part_header::EditablePartHeader;
use crate::part::editable_part_object::EditablePartObject;

/// A tool's part saved as text (`EScene::SaveToolLTX`): its scene's GUID, the tool's `[main]` and `[modif]`, and one
/// `[object_N]` section per object with the sections named after it. Scene objects, spawns, lights, sectors, portals,
/// ways and the other object tools save this way.
///
/// The file is kept whole as the [`IniFile`] it was read as, so it writes back byte for byte; a spawn's data is the
/// `[object_N_spawndata]` section's numbered fields, one per value its packet holds (`SIniFileStream`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditableTextPart {
  pub ini: IniFile,
}

impl EditableTextPart {
  /// Reads a text part and checks it carries a GUID, as `EScene::LoadLevelPartLTX` does before reading on.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a file [`IniFormat::XRAY_SAVE_AS`] reads, or the GUID is missing or
  /// malformed.
  pub fn read_from_bytes(bytes: &[u8]) -> XrfResult<Self> {
    let part: Self = Self {
      ini: IniFile::read_from_bytes_as(bytes, &IniFormat::XRAY_SAVE_AS)?,
    };

    part.get_guid()?;

    Ok(part)
  }

  /// # Errors
  ///
  /// Returns an error when the file would not read back as itself or does not encode as Windows-1251.
  pub fn write_to_bytes(&self) -> XrfResult<Vec<u8>> {
    self.ini.write_to_bytes_as(&IniFormat::XRAY_SAVE_AS)
  }

  /// # Errors
  ///
  /// Returns an error when the `[guid]` section is missing or malformed.
  pub fn get_guid(&self) -> XrfResult<EditableLevelGuid> {
    EditableLevelGuid::read_from_section(self.ini.section_required(EditableLevelGuid::SECTION)?)
  }

  pub fn set_guid(&mut self, guid: &EditableLevelGuid) {
    guid.write_to_section(self.ini.section_mut(EditableLevelGuid::SECTION));
  }

  /// The tool's `[main]`, its version under `version_key` ([`EditablePartHeader::VERSION_KEY`] for most tools); none
  /// for a part that writes no objects.
  ///
  /// # Errors
  ///
  /// Returns an error when the section is present but malformed.
  pub fn get_header(&self, version_key: &str) -> XrfResult<Option<EditablePartHeader>> {
    self
      .ini
      .section(EditablePartHeader::SECTION)
      .map(|section| EditablePartHeader::read_from_section(section, version_key))
      .transpose()
  }

  /// The objects `[main]` counts, each with the sections named after it, gathered in one pass over the sections: a
  /// decompiled outdoor level holds tens of thousands.
  ///
  /// # Errors
  ///
  /// Returns an error when `[main]`'s count is malformed or an object it counts has no section.
  pub fn get_objects(&self) -> XrfResult<Vec<EditablePartObject<'_>>> {
    let Some(main) = self.ini.section(EditablePartHeader::SECTION) else {
      return Ok(Vec::new());
    };
    let count: usize = main.get_parsed::<u32>(EditablePartHeader::OBJECTS_COUNT_KEY)? as usize;
    let mut sections: Vec<Option<&IniSection>> = vec![None; count];
    let mut children: Vec<Vec<&IniSection>> = vec![Vec::new(); count];

    for section in &self.ini.sections {
      let Some(rest) = section.name.strip_prefix(EditablePartObject::SECTION_PREFIX) else {
        continue;
      };
      let digits: &str = rest.split('_').next().unwrap_or_default();
      let Some(index) = digits
        .parse::<usize>()
        .ok()
        .filter(|index| *index < count && index.to_string() == digits)
      else {
        continue;
      };

      if rest.len() == digits.len() {
        sections[index] = Some(section);
      } else {
        children[index].push(section);
      }
    }

    sections
      .into_iter()
      .zip(children)
      .enumerate()
      .map(|(index, (section, children))| {
        let section: &IniSection = section.ok_or_else(|| {
          XrfError::new_parsing_error(format!(
            "Part counts {count} objects but has no section [{}]",
            EditablePartObject::get_section_name(index)
          ))
        })?;

        Ok(EditablePartObject {
          index,
          section,
          children,
        })
      })
      .collect()
  }
}
