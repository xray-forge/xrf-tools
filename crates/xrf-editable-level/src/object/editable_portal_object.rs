use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};
use xrf_math::Vector3d;

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A portal between two sectors, `CPortal`: an object of `portal.part`, a polygon the renderer sees one sector through
/// from the other.
#[derive(Clone, Debug, PartialEq)]
pub struct EditablePortalObject {
  pub object: EditableCustomObject,
  /// The sector the portal faces, by its name in `sector.part`.
  pub sector_front: String,
  pub sector_back: String,
  /// The polygon's corners in world space, in order.
  pub vertices: Vec<Vector3d>,
}

impl EditablePortalObject {
  /// `PORTAL_VERSION`, the only one `CPortal::LoadLTX` reads.
  pub const VERSION: u32 = 0x10;

  /// The fewest corners `LoadLTX` takes for a portal.
  pub const MIN_VERTICES: usize = 3;

  fn get_vertex_key(index: usize) -> String {
    format!("vertex_{index:04}")
  }
}

impl EditableToolObject for EditablePortalObject {
  /// `OBJCLASS_PORTAL`.
  const CLASS_ID: i32 = 9;
  const TOOL_NAME: &'static str = "portal";
  const KEEPS_FLAGS: bool = true;

  /// As `CPortal::LoadLTX` reads it (`portal.cpp:497`), refusing another version or fewer than three corners as it
  /// does.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Portal [{}] is of version {version}, not {}",
        section.name,
        Self::VERSION
      )));
    }

    let count: usize = EditableLevelValue::read_u32(section, "vert_count")? as usize;

    if count < Self::MIN_VERTICES {
      return Err(XrfError::new_parsing_error(format!(
        "Portal [{}] has {count} corners, fewer than {}",
        section.name,
        Self::MIN_VERTICES
      )));
    }

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      sector_front: section.get_required("sector_front")?.to_owned(),
      sector_back: section.get_required("sector_back")?.to_owned(),
      vertices: (0..count)
        .map(|index| EditableLevelValue::read_vector3(section, &Self::get_vertex_key(index)))
        .collect::<XrfResult<Vec<Vector3d>>>()?,
    })
  }

  /// As `CPortal::SaveLTX` writes it (`portal.cpp:539`), at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("sector_front", self.sector_front.clone());
    section.set("sector_back", self.sector_back.clone());
    section.set("vert_count", self.vertices.len().to_string());

    for (index, vertex) in self.vertices.iter().enumerate() {
      section.set(Self::get_vertex_key(index), EditableLevelValue::format_vector3(vertex));
    }
  }
}
