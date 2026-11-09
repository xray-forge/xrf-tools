use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// One mesh a sector holds, `CSectorItem`: a scene object by name, and one of its object's meshes by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableSectorItem {
  pub object: String,
  pub mesh: String,
}

/// A sector, `CSector`: an object of `sector.part`, the meshes the renderer culls together, joined to other sectors by
/// portals.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableSectorObject {
  pub object: EditableCustomObject,
  /// The colour the editor draws it in: red, green, blue and alpha bytes.
  pub color: [u8; 4],
  /// Whether it is the sector everything no other sector holds falls into.
  pub is_default: bool,
  pub items: Vec<EditableSectorItem>,
  /// The level map index the sector switches to, 255 for none (`m_map_idx`).
  pub map_index: u8,
}

impl EditableSectorObject {
  /// `SECTOR_VERSION`, which the editor writes.
  pub const VERSION: u32 = 0x12;

  /// The oldest version `CSector::LoadLTX` reads; it keeps no map index.
  pub const MIN_VERSION: u32 = 0x11;

  /// The map index of a sector that switches to none.
  pub const NO_MAP: u8 = u8::MAX;

  fn read_color(section: &IniSection) -> XrfResult<[u8; 4]> {
    let value: &str = section.get_required("sector_color")?;
    let channels: Vec<u8> = value
      .split(',')
      .map(|channel| channel.trim().parse::<u8>())
      .collect::<Result<Vec<u8>, _>>()
      .map_err(|error| {
        XrfError::new_parsing_error(format!(
          "[{}] sector_color '{value}' is not four bytes: {error}",
          section.name
        ))
      })?;

    channels.try_into().map_err(|channels: Vec<u8>| {
      XrfError::new_parsing_error(format!(
        "[{}] sector_color '{value}' holds {} channels, not four",
        section.name,
        channels.len()
      ))
    })
  }
}

impl EditableToolObject for EditableSectorObject {
  /// `OBJCLASS_SECTOR`.
  const CLASS_ID: i32 = 8;
  const TOOL_NAME: &'static str = "sector";
  const KEEPS_FLAGS: bool = true;

  /// As `CSector::LoadLTX` reads it (`sector.cpp:525`), refusing a version before 17, or a sector of no item, which it
  /// does not load.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;
    let count: u32 = EditableLevelValue::read_u32(section, "items_count")?;

    if version < Self::MIN_VERSION || count == 0 {
      return Err(XrfError::new_parsing_error(format!(
        "Sector [{}] is of version {version} with {count} items, where the editor loads version {} or later with \
         one or more",
        section.name,
        Self::MIN_VERSION
      )));
    }

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      color: Self::read_color(section)?,
      is_default: EditableLevelValue::read_bool(section, "default")?,
      items: (0..count)
        .map(|index| {
          Ok(EditableSectorItem {
            object: section
              .get_required(&format!("item_object_name_{index:04}"))?
              .to_owned(),
            mesh: section.get_required(&format!("item_mesh_name_{index:04}"))?.to_owned(),
          })
        })
        .collect::<XrfResult<Vec<EditableSectorItem>>>()?,
      map_index: if version >= Self::VERSION {
        section.get_parsed::<u8>("change_map_to_idx")?
      } else {
        Self::NO_MAP
      },
    })
  }

  /// As `CSector::SaveLTX` writes it (`sector.cpp:556`), at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);
    let [red, green, blue, alpha] = self.color;

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("sector_color", format!("{red}, {green}, {blue}, {alpha}"));
    section.set("default", EditableLevelValue::format_bool(self.is_default));
    section.set("items_count", self.items.len().to_string());

    for (index, item) in self.items.iter().enumerate() {
      section.set(format!("item_object_name_{index:04}"), item.object.clone());
      section.set(format!("item_mesh_name_{index:04}"), item.mesh.clone());
    }

    section.set("change_map_to_idx", self.map_index.to_string());
  }
}
